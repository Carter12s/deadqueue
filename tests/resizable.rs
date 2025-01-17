#[cfg(feature = "resizable")]
mod tests {

    use std::sync::Arc;

    use deadqueue::resizable::Queue;

    #[tokio::test]
    async fn test_basics() {
        let queue: Queue<usize> = Queue::new(2);
        assert_eq!(queue.len(), 0);
        assert!(queue.try_push(1).is_ok());
        assert_eq!(queue.len(), 1);
        assert!(queue.try_push(2).is_ok());
        assert_eq!(queue.len(), 2);
        assert!(queue.try_push(3).is_err());
        assert_eq!(queue.len(), 2);
        assert!(queue.try_pop().is_some());
        assert_eq!(queue.len(), 1);
        assert!(queue.try_push(3).is_ok());
        assert_eq!(queue.len(), 2);
    }

    #[tokio::test]
    async fn test_parallel() {
        let queue: Arc<Queue<usize>> = Arc::new(Queue::new(10));
        let mut futures = Vec::new();
        for _ in 0..100usize {
            let queue = queue.clone();
            futures.push(tokio::spawn(async move {
                for _ in 0..100usize {
                    queue.pop().await;
                }
            }));
        }
        for _ in 0..100usize {
            let queue = queue.clone();
            futures.push(tokio::spawn(async move {
                for i in 0..100usize {
                    queue.push(i).await;
                }
            }));
        }
        for future in futures {
            future.await.unwrap();
        }
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_debug() {
        struct NoDebug {}
        let queue: Queue<NoDebug> = Queue::new(1);
        format!("{:?}", queue);
    }

    #[tokio::test]
    async fn test_resize_enlarge() {
        let queue: Queue<usize> = Queue::new(0);
        queue.resize(1).await;
        assert_eq!(queue.capacity(), 1);
    }

    #[tokio::test]
    async fn test_resize_shrink() {
        let queue: Queue<usize> = Queue::new(2);
        queue.try_push(0).unwrap();
        queue.resize(1).await;
        assert_eq!(queue.capacity(), 1);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.try_push(42), Err(42));
        queue.resize(0).await;
        assert_eq!(queue.capacity(), 0);
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.try_push(42), Err(42));
    }

    #[tokio::test]
    async fn test_is_full_basic() {
        let queue: Queue<usize> = Queue::new(2);
        assert!(!queue.is_full(), "Should be empty at construction");
        queue.push(1).await;
        assert!(
            !queue.is_full(),
            "Should not be full a one less than capacity"
        );
        queue.push(2).await;
        assert!(queue.is_full(), "Should now be full");
        let _ = queue.pop().await;
        assert!(!queue.is_full(), "Should no longer be full after pop");
    }
}
