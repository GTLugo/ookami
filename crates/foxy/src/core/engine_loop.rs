use std::{
  marker::PhantomData,
  sync::{Arc, Barrier},
};

use foxy_renderer::renderer::{render_data::RenderData, Renderer};
use foxy_types::{behavior::Polling, thread::EngineThread};
use foxy_util::{log::LogErr, time::EngineTime};
use foxy_window::prelude::*;
use messaging::Mailbox;
use tracing::*;

use super::{engine::Foxy, stage::StageDiscriminants};
use crate::core::{
  builder::{FoxyBuilder, FoxyCreateInfo, HasSize, HasTitle, MissingSize, MissingTitle},
  message::{GameLoopMessage, RenderLoopMessage},
  render_loop::RenderLoop,
  stage::Stage,
  time::Time,
};

pub struct Framework<'a> {
  polling_strategy: Polling,
  render_thread: EngineThread<RenderLoop>,
  game_mailbox: Mailbox<GameLoopMessage, RenderLoopMessage>,
  sync_barrier: Arc<Barrier>,

  current_stage: StageDiscriminants,
  current_message: WindowMessage,

  foxy: Foxy,

  _phantom: PhantomData<&'a ()>,
}

impl Framework<'_> {
  pub fn builder() -> FoxyBuilder<MissingTitle, MissingSize> {
    Default::default()
  }

  pub(crate) fn new(create_info: FoxyCreateInfo<HasTitle, HasSize>) -> anyhow::Result<Self> {
    trace!("Firing up Foxy");

    // TODO: make this adjustable
    let time = Time::new(128.0, 1024);
    let render_time = EngineTime::new(128.0, 1024);

    let mut window = Window::builder()
      .with_title(create_info.title.0)
      .with_size(create_info.size.width, create_info.size.height)
      .with_color_mode(create_info.color_mode)
      .with_close_behavior(create_info.close_behavior)
      .with_visibility(Visibility::Hidden)
      .build()?;

    let renderer = Renderer::new(&window, window.inner_size())?;
    window.set_visibility(Visibility::Shown);

    let sync_barrier = Arc::new(Barrier::new(2));

    let (renderer_mailbox, game_mailbox) = Mailbox::new_entangled_pair();
    let render_thread = EngineThread::new(RenderLoop {
      renderer,
      messenger: renderer_mailbox,
      sync_barrier: sync_barrier.clone(),
      time: render_time,
    });

    let current_stage = StageDiscriminants::Initialize;
    let foxy = Foxy::new(time, window);

    Ok(Self {
      current_stage,
      render_thread,
      game_mailbox,
      sync_barrier,
      polling_strategy: create_info.polling_strategy,
      foxy,
      current_message: WindowMessage::None,
      _phantom: PhantomData,
    })
  }

  pub fn foxy(&mut self) -> &mut Foxy {
    &mut self.foxy
  }

  fn next_window_message(&mut self) -> Option<WindowMessage> {
    if let Polling::Wait = self.polling_strategy {
      self.foxy.window.wait()
    } else {
      self.foxy.window.next()
    }
  }

  fn next_state(&mut self) -> Option<Stage<'_>> {
    /*
     * NOTE: each stage in the match is the PREVIOUS stage!!!
     *       I've written the ACTUAL stage at the top of each
     */
    let new_state = match self.current_stage {
      StageDiscriminants::Initialize => {
        // Start
        info!("KON KON KITSUNE!");
        self.render_thread.run(());
        Stage::Start { foxy: &mut self.foxy }
      }
      StageDiscriminants::Start => {
        // Begin Frame / Exiting
        if let Some(message) = self.next_window_message() {
          self.current_message = message;
          Stage::BeginFrame {
            foxy: &mut self.foxy,
            message: &mut self.current_message,
          }
        } else {
          Stage::Exiting { foxy: &mut self.foxy }
        }
      }
      StageDiscriminants::BeginFrame => {
        // Early Update
        self.sync_barrier.wait();
        self.foxy.time.update();

        Stage::EarlyUpdate {
          foxy: &mut self.foxy,
          message: &mut self.current_message,
        }
      }
      StageDiscriminants::EarlyUpdate => {
        // Fixed Update / Update
        if self.foxy.time.should_do_tick() {
          self.foxy.time.tick();
          Stage::FixedUpdate { foxy: &mut self.foxy }
        } else {
          Stage::Update {
            foxy: &mut self.foxy,
            message: &mut self.current_message,
          }
        }
      }
      StageDiscriminants::FixedUpdate => {
        // Fixed Update / Update
        if self.foxy.time.should_do_tick() {
          self.foxy.time.tick();
          Stage::FixedUpdate { foxy: &mut self.foxy }
        } else {
          Stage::Update {
            foxy: &mut self.foxy,
            message: &mut self.current_message,
          }
        }
      }
      StageDiscriminants::Update => {
        // End Frame
        match self
          .game_mailbox
          .send_and_wait(GameLoopMessage::RenderData(RenderData {}))
          .log_error()
        {
          Ok(render_response) => Stage::EndFrame {
            foxy: &mut self.foxy,
            message: &mut self.current_message,
            render_response,
          },
          Err(_) => Stage::Exiting { foxy: &mut self.foxy },
        }
      }
      StageDiscriminants::EndFrame => {
        // Begin Frame / Exiting
        if let Some(message) = self.next_window_message() {
          self.current_message = message;
          Stage::BeginFrame {
            foxy: &mut self.foxy,
            message: &mut self.current_message,
          }
        } else {
          Stage::Exiting { foxy: &mut self.foxy }
        }
      }
      StageDiscriminants::Exiting => {
        // Exit Loop
        let _ = self.game_mailbox.send(GameLoopMessage::Exit).log_error();
        self.sync_barrier.wait();

        self.render_thread.join();
        Stage::ExitLoop
      }
      StageDiscriminants::ExitLoop => {
        // Never gets sent to clients
        info!("OTSU KON DESHITA!");
        return None;
      }
    };

    self.current_stage = StageDiscriminants::from(&new_state);

    Some(new_state)
  }
}

impl<'a> Iterator for Framework<'a> {
  type Item = Stage<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    // it is irrefutable that the reference not outlive Foxy
    unsafe { std::mem::transmute(self.next_state()) }
  }
}
