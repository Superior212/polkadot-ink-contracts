#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod todo_list {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;

    #[derive(scale::Encode, scale::Decode, Default, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct TodoItem {
        pub description: String,
        pub completed: bool,
    }

    #[ink(storage)]
    pub struct TodoList {
        items: Vec<TodoItem>,
        owner: AccountId,
    }

    impl TodoList {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                items: Vec::new(),
                owner: Self::env().caller(),
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        #[ink(message)]
        pub fn add_item(&mut self, description: String) {
            self.items.push(TodoItem {
                description,
                completed: false,
            });
        }

        #[ink(message)]
        pub fn get_items(&self) -> Vec<TodoItem> {
            self.items.clone()
        }

        #[ink(message)]
        pub fn mark_completed(&mut self, index: u32) {
            if let Some(item) = self.items.get_mut(index as usize) {
                item.completed = true;
            }
        }
        
        #[ink(message)]
        pub fn clear_completed(&mut self) {
            self.items.retain(|item| !item.completed);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let todo_list = TodoList::default();
            assert_eq!(todo_list.get_items().len(), 0);
        }

        #[ink::test]
        fn add_item_works() {
            let mut todo_list = TodoList::new();
            todo_list.add_item("write tests".into());
            assert_eq!(todo_list.get_items().len(), 1);
            assert_eq!(todo_list.get_items()[0].description, "write tests");
            assert!(!todo_list.get_items()[0].completed);
        }

        #[ink::test]
        fn mark_completed_works() {
            let mut todo_list = TodoList::new();
            todo_list.add_item("write tests".into());
            todo_list.mark_completed(0);
            assert!(todo_list.get_items()[0].completed);
        }

        #[ink::test]
        fn clear_completed_works() {
            let mut todo_list = TodoList::new();
            todo_list.add_item("write tests".into());
            todo_list.add_item("deploy contract".into());
            todo_list.mark_completed(0);
            todo_list.clear_completed();
            assert_eq!(todo_list.get_items().len(), 1);
            assert_eq!(todo_list.get_items()[0].description, "deploy contract");
        }
    }
}
