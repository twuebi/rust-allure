mod some;

#[cfg(test)]
mod test {
    use crate::some::REPORTER;

    #[tokio::test]
    async fn test() {
        let lock = REPORTER.get_or_init(Default::default()).await.lock().await;
    }
}
