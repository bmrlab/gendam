#[cfg(test)]
mod tests {
    use crate::db::shared::test::setup;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_full_text_search_with_highlight() {
        let db = setup(None).await;
        let res = db
            .full_text_search_with_highlight(vec!["LVL小河板".to_string()])
            .await
            .unwrap();
        println!("res: {res:#?}");
    }
}
