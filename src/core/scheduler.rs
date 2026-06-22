use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use log::{info, warn, error};

/// Task priority levels for the scheduler
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Critical = 0,  // Camera capture, dispatch
    High = 1,      // Gesture classification
    Normal = 2,    // Spatial processing
    Low = 3,       // Logging, visualization
    Background = 4, // Calibration, analytics
}

/// A scheduled task with priority
pub struct ScheduledTask {
    pub name: String,
    pub priority: TaskPriority,
    pub task: Box<dyn FnOnce() + Send + Sync>,
}

/// Task scheduler for managing concurrent operations
pub struct TaskScheduler {
    tasks: Arc<Mutex<Vec<ScheduledTask>>>,
    handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    running: Arc<Mutex<bool>>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            handles: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Schedule a task with priority
    pub async fn schedule<F>(&self, name: &str, priority: TaskPriority, task: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let scheduled = ScheduledTask {
            name: name.to_string(),
            priority,
            task: Box::new(task),
        };
        
        let mut tasks = self.tasks.lock().await;
        tasks.push(scheduled);
        // Sort by priority (higher priority = lower number)
        tasks.sort_by_key(|t| t.priority);
        info!("Scheduled task: {} with priority {:?}", name, priority);
    }

    /// Execute all scheduled tasks
    pub async fn execute(&self) {
        let mut running = self.running.lock().await;
        if *running {
            warn!("Scheduler already running");
            return;
        }
        *running = true;
        drop(running);

        let tasks = self.tasks.lock().await;
        let mut handles = self.handles.lock().await;

        for task in tasks.iter() {
            let name = task.name.clone();
            let task_fn = std::mem::take(&mut *task.task.box_clone?);
            
            let handle = tokio::spawn(async move {
                info!("Executing task: {}", name);
                task_fn();
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles.iter() {
            if let Err(e) = handle.await {
                error!("Task failed: {}", e);
            }
        }

        let mut running = self.running.lock().await;
        *running = false;
    }

    /// Clear all scheduled tasks
    pub async fn clear(&self) {
        let mut tasks = self.tasks.lock().await;
        tasks.clear();
        info!("Cleared all scheduled tasks");
    }

    /// Get number of pending tasks
    pub async fn pending_count(&self) -> usize {
        self.tasks.lock().await.len()
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler() {
        let scheduler = TaskScheduler::new();
        
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        
        scheduler.schedule("test_task", TaskPriority::Normal, move || {
            let mut count = counter_clone.blocking_lock();
            *count += 1;
        }).await;
        
        assert_eq!(scheduler.pending_count().await, 1);
        
        scheduler.execute().await;
        assert_eq!(*counter.lock().await, 1);
    }
}