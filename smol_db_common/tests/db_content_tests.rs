#[cfg(test)]
mod tests {
    use smol_db_common::db_content::DBContent;
    use smol_db_common::db_packets::db_keyed_list_location::DBKeyedListLocation;

    #[test]
    fn test_list_made_empty_by_removal() {
        let mut content = DBContent::default();
        let list_name = "list 2";
        let data = "uncool data";

        content.add_data_to_list(&list_name.into(), data.into());
        content.add_data_to_list(&list_name.into(), data.into());

        assert_eq!(
            content.remove_data_from_list(&list_name.into()).unwrap(),
            data.to_string()
        );
        assert_eq!(
            content.remove_data_from_list(&list_name.into()).unwrap(),
            data.to_string()
        );
        let fail_removal = content.remove_data_from_list(&list_name.into());
        assert!(fail_removal.is_none());

        assert!(content.get_length_of_list(&list_name.into()).is_none()); // list no longer exists since it should have been removed
    }

    #[test]
    fn test_adding_and_clearing() {
        let mut content = DBContent::default();
        let list_name = "list 2";
        let data = "uncool data";

        content.add_data_to_list(&list_name.into(), data.into());
        content.add_data_to_list(&list_name.into(), data.into());

        assert_eq!(content.get_length_of_list(&list_name.into()).unwrap(), 2);

        assert!(content.clear_list(&list_name.into()));
        assert!(content.get_length_of_list(&list_name.into()).is_none());
        assert!(!content.clear_list(&list_name.into()));
    }

    #[test]
    fn test_remove_data_from_list() {
        let mut content = DBContent::default();
        let list_name = "list 1";
        let data = "cool data";

        content.add_data_to_list(&list_name.into(), data.into());

        {
            let length = content.get_length_of_list(&DBKeyedListLocation::new(None, list_name));
            assert_eq!(length.unwrap(), 1);
        }

        content.add_data_to_list(&list_name.into(), data.into());

        {
            let length = content.get_length_of_list(&DBKeyedListLocation::new(None, list_name));
            assert_eq!(length.unwrap(), 2);
        }

        assert_eq!(
            content.remove_data_from_list(&list_name.into()).unwrap(),
            data.to_string()
        );

        {
            let length = content.get_length_of_list(&DBKeyedListLocation::new(None, list_name));
            assert_eq!(length.unwrap(), 1);
        }

        assert!(content
            .remove_data_from_list(&DBKeyedListLocation::new(Some(99999), list_name))
            .is_none());
    }
}
