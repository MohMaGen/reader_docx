use std::future::Future;
use std::{
    ops::Deref,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex,
    },
    task::Context,
    thread::{self, JoinHandle},
};

use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};

use crate::{display_error, AsAnyhow, Command, Message, State, StateMutex};

pub fn spawn(state: Arc<Mutex<State>>, commands: Arc<Mutex<Vec<Command>>>) -> JoinHandle<()> {
    thread::spawn(move || loop {
        match apply_loop(Arc::clone(&state), Arc::clone(&commands)) {
            Ok(true) => break,
            Err(err) => display_error(&err),
            _ => {}
        }
    })
}

pub fn process_message(state: Arc<Mutex<State>>, message: &Message) -> anyhow::Result<()> {
    match message {
        Message::LoadDocx(result) => match result.deref() {
            Ok(document) => {
                #[cfg(debug_assertions)]
                println!("{}", document.docx_document);
                state.lock().as_anyhow()?.document = Some(Arc::new(Box::new(document.clone())));
            }
            _ => {}
        },
        _ => println!("Aboba"),
    }
    Ok(())
}


pub fn apply_loop(
    state: Arc<Mutex<State>>,
    commands: Arc<Mutex<Vec<Command>>>,
) -> anyhow::Result<bool> {
    if Arc::clone(&state).should_exit()? {
        return Ok(true);
    }

    let mut commands_buf = Vec::new();
    {
        let mut commands = commands.lock().as_anyhow()?;
        commands_buf.append(&mut commands);
    }

    for command in commands_buf {
        let state = Arc::clone(&state);
        thread::spawn(move || {
            let (executor, spawner) = new_executor_and_spawner();

            spawner.spawn(async move {
                match command.await {
                    Ok(message) => match process_message(Arc::clone(&state), &message) {
                        Err(err) => display_error(&err),
                        _ => {}
                    },
                    Err(err) => display_error(&err),
                }
            });

            drop(spawner);

            executor.run();
        });
    }

    Ok(false)
}

struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    task_sender: SyncSender<Arc<Task>>,
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = Arc::clone(&arc_self);
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many commands");
    }
}

impl Executor {
    fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                if future.as_mut().poll(context).is_pending() {
                    *future_slot = Some(future);
                }
            }
        }
    }
}
