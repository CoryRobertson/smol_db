#[cfg(test)]
#[cfg(feature = "async")]
mod tests {
    use smol_db_client::SmolDbClient;
    use smol_db_common::prelude::DBSettings;
    use std::time::Duration;

    const TESTING_IP: &str = "localhost:8222";
    const TESTING_KEY: &str = "test_key_123";

    async fn get_client_and_set_key() -> SmolDbClient {
        let mut client = SmolDbClient::new(TESTING_IP).await.unwrap();
        assert!(client.set_access_key(TESTING_KEY.to_string()).await.is_ok());
        client
    }

    #[tokio::test]
    async fn test_client_connect() {
        let mut client = get_client_and_set_key().await;

        let f1 = client
            .create_db("async_connect", DBSettings::default())
            .await;

        let f2 = client.delete_db("async_connect").await;

        assert!(f1.is_ok());

        assert!(f2.is_ok());

        assert!(client.disconnect().await.is_ok());
    }

    #[tokio::test]
    async fn test_client_write_read_db() {
        let mut client = get_client_and_set_key().await;

        const DB_NAME: &str = "async_test_write_read";

        assert!(client
            .create_db(DB_NAME, DBSettings::default())
            .await
            .is_ok());

        assert!(client.get_role(DB_NAME).await.unwrap().is_admin());

        assert!(client.write_db(DB_NAME, "loc1", "d1").await.is_ok());

        assert_eq!(
            client
                .read_db(DB_NAME, "loc1")
                .await
                .unwrap()
                .into_option()
                .unwrap(),
            "d1".to_string()
        );

        assert!(client.delete_db(DB_NAME).await.is_ok());

        assert!(client.disconnect().await.is_ok());
    }

    #[tokio::test]
    async fn test_setup_encryption() {
        let mut client = get_client_and_set_key().await;

        const DB_NAME: &str = "async_test_encryption";

        assert!(client.setup_encryption().await.is_ok());

        assert!(client.is_encryption_enabled());

        assert!(client
            .create_db(DB_NAME, DBSettings::default())
            .await
            .is_ok());

        assert!(client.get_role(DB_NAME).await.unwrap().is_admin());

        assert!(client.write_db(DB_NAME, "loc1", "d1").await.is_ok());

        assert_eq!(
            client
                .read_db(DB_NAME, "loc1")
                .await
                .unwrap()
                .into_option()
                .unwrap(),
            "d1".to_string()
        );

        assert!(client.delete_db(DB_NAME).await.is_ok());

        assert!(client.disconnect().await.is_ok());
    }

    #[tokio::test]
    async fn test_reconnect() {
        let mut client = get_client_and_set_key().await;

        assert!(client.disconnect().await.is_ok());

        assert!(client.reconnect().await.is_ok());
    }

    #[tokio::test]
    async fn test_delete_data() {
        let mut client = get_client_and_set_key().await;

        const DB_NAME: &str = "async_test_delete_data";

        assert!(client.setup_encryption().await.is_ok());

        assert!(client.is_encryption_enabled());

        assert!(client
            .create_db(DB_NAME, DBSettings::default())
            .await
            .is_ok());

        assert!(client.get_role(DB_NAME).await.unwrap().is_admin());

        assert!(client.write_db(DB_NAME, "loc1", "d1").await.is_ok());

        assert_eq!(
            client
                .read_db(DB_NAME, "loc1")
                .await
                .unwrap()
                .into_option()
                .unwrap(),
            "d1".to_string()
        );

        assert!(client.delete_data(DB_NAME, "loc1").await.is_ok());

        assert!(client.read_db(DB_NAME, "loc1").await.is_err());

        assert!(client.delete_db(DB_NAME).await.is_ok());

        assert!(client.disconnect().await.is_ok());
    }

    #[tokio::test]
    #[cfg(feature = "statistics")]
    async fn test_get_stats() {
        let mut client = get_client_and_set_key().await;

        const DB_NAME: &str = "async_test_stats";

        assert!(client
            .create_db(DB_NAME, DBSettings::default())
            .await
            .is_ok());

        assert!(client.get_role(DB_NAME).await.unwrap().is_admin());

        assert!(client.write_db(DB_NAME, "loc1", "d1").await.is_ok());

        assert!(client.get_stats(DB_NAME).await.is_ok());

        assert!(client.delete_db(DB_NAME).await.is_ok());
    }

    #[tokio::test]
    async fn test_get_settings() {
        let mut client = get_client_and_set_key().await;

        const DB_NAME: &str = "async_test_settings";

        const SETTINGS: DBSettings = DBSettings::new(
            Duration::from_secs(22),
            (false, true, true),
            (true, false, false),
            vec![],
            vec![],
        );

        assert!(client.create_db(DB_NAME, SETTINGS).await.is_ok());

        assert!(client.get_role(DB_NAME).await.unwrap().is_admin());

        assert_eq!(client.get_db_settings(DB_NAME).await.unwrap(), SETTINGS);

        assert!(client
            .set_db_settings(DB_NAME, DBSettings::default())
            .await
            .is_ok());

        assert_eq!(
            client.get_db_settings(DB_NAME).await.unwrap(),
            DBSettings::default()
        );

        assert!(client.delete_db(DB_NAME).await.is_ok());
    }

    #[tokio::test]
    async fn test_list_db() {
        let mut client = get_client_and_set_key().await;

        const DB_NAME: &str = "async_test_list_db";

        assert!(client
            .create_db(DB_NAME, DBSettings::default())
            .await
            .is_ok());

        assert!(client.list_db().await.unwrap().len() >= 1);

        assert!(client.write_db(DB_NAME, "loc1", "d1").await.is_ok());
        assert!(client.write_db(DB_NAME, "loc2", "d2").await.is_ok());

        assert_eq!(client.list_db_contents(DB_NAME).await.unwrap().len(), 2);

        assert!(client.delete_db(DB_NAME).await.is_ok());
    }
}
