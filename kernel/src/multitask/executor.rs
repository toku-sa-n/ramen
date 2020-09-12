// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::task::{self, Task},
    alloc::{collections::BTreeMap, sync::Arc, task::Wake},
    core::task::{Context, Poll, Waker},
    crossbeam_queue::ArrayQueue,
    x86_64::instructions::interrupts::{self, enable_interrupts_and_hlt},
};

pub struct Executor {
    tasks: BTreeMap<task::Id, Task>,
    woken_id_queue: Arc<ArrayQueue<task::Id>>,
    waker_cache: BTreeMap<task::Id, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            woken_id_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let id = task.id();
        if self.tasks.insert(id, task).is_some() {
            panic!("Task ID confliction!");
        }

        self.woken_id_queue
            .push(id)
            .expect("woken_id_queue is full");
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_woken_tasks();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        interrupts::disable();
        if self.woken_id_queue.is_empty() {
            enable_interrupts_and_hlt()
        } else {
            interrupts::enable()
        }
    }

    fn run_woken_tasks(&mut self) {
        let Self {
            tasks,
            woken_id_queue,
            waker_cache,
        } = self;

        while let Ok(id) = woken_id_queue.pop() {
            let task = match tasks.get_mut(&id) {
                Some(task) => task,
                None => continue,
            };

            let waker = waker_cache
                .entry(id)
                .or_insert_with(|| TaskWaker::create_waker(id, woken_id_queue.clone()));

            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(_) => {
                    tasks.remove(&id);
                    waker_cache.remove(&id);
                }
                Poll::Pending => {}
            }
        }
    }
}

struct TaskWaker {
    id: task::Id,
    task_queue: Arc<ArrayQueue<task::Id>>,
}

impl TaskWaker {
    fn create_waker(id: task::Id, task_queue: Arc<ArrayQueue<task::Id>>) -> Waker {
        Waker::from(Arc::new(Self { id, task_queue }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.id).expect("task_queue is full")
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task()
    }
}
