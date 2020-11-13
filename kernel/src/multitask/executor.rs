// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::task,
    alloc::{collections::BTreeMap, rc::Rc},
    core::{
        cell::RefCell,
        task::{Context, Poll, Waker},
    },
    x86_64::instructions::interrupts,
};

pub struct Executor {
    task_collection: Rc<RefCell<task::Collection>>,
    waker_collection: BTreeMap<task::Id, Waker>,
}

impl Executor {
    pub fn new(task_collection: Rc<RefCell<task::Collection>>) -> Self {
        Self {
            task_collection,
            waker_collection: BTreeMap::new(),
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_woken_tasks();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        interrupts::disable();
        if self.task_collection.borrow().woken_task_exists() {
            interrupts::enable()
        } else {
            interrupts::enable_and_hlt()
        }
    }

    fn run_woken_tasks(&mut self) {
        while let Some(id) = self.pop_woken_task_id() {
            self.run_task(id);
        }
    }

    fn pop_woken_task_id(&mut self) -> Option<task::Id> {
        self.task_collection.borrow_mut().pop_woken_task_id()
    }

    fn run_task(&mut self, id: task::Id) {
        let Self {
            task_collection,
            waker_collection,
        } = self;

        let mut task = match task_collection.borrow_mut().remove_task(id) {
            Some(task) => task,
            None => return,
        };

        let waker = waker_collection
            .entry(id)
            .or_insert_with(|| task_collection.borrow_mut().create_waker(id));

        let mut context = Context::from_waker(waker);
        match task.poll(&mut context) {
            Poll::Ready(_) => {
                self.task_collection.borrow_mut().remove_task(id);
                self.waker_collection.remove(&id);
            }
            Poll::Pending => self.task_collection.borrow_mut().add_task_as_sleep(task),
        }
    }
}
