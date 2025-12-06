// State Reducer Pattern for State Management
// Security: Layer 10 - API Security
// Predictable state management with action dispatching and middleware support

use std::collections::HashMap;
use std::sync::Arc;
use std::any::Any;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

/// Action trait for state updates
pub trait Action: Any + Send + Sync + std::fmt::Debug {
    fn action_type(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}

/// Reducer function type
pub type ReducerFn<S> = Arc<dyn Fn(&S, &dyn Action) -> S + Send + Sync>;

/// Middleware function type
pub type MiddlewareFn<S> = Arc<dyn Fn(&S, &dyn Action, &dyn Fn(&dyn Action)) + Send + Sync>;

/// Subscriber function type
pub type SubscriberFn<S> = Arc<dyn Fn(&S) + Send + Sync>;

/// State store with reducer pattern
pub struct Store<S: Clone + Send + Sync> {
    state: Arc<RwLock<S>>,
    reducer: ReducerFn<S>,
    middlewares: Vec<MiddlewareFn<S>>,
    subscribers: Arc<RwLock<Vec<SubscriberFn<S>>>>,
}

impl<S: Clone + Send + Sync + 'static> Store<S> {
    /// Create a new store with initial state and reducer
    pub fn new(initial_state: S, reducer: ReducerFn<S>) -> Self {
        Self {
            state: Arc::new(RwLock::new(initial_state)),
            reducer,
            middlewares: Vec::new(),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add middleware
    pub fn add_middleware(&mut self, middleware: MiddlewareFn<S>) {
        self.middlewares.push(middleware);
    }

    /// Subscribe to state changes
    pub fn subscribe(&self, subscriber: SubscriberFn<S>) -> usize {
        let mut subscribers = self.subscribers.write();
        subscribers.push(subscriber);
        subscribers.len() - 1
    }

    /// Unsubscribe from state changes
    pub fn unsubscribe(&self, index: usize) {
        let mut subscribers = self.subscribers.write();
        if index < subscribers.len() {
            subscribers.remove(index);
        }
    }

    /// Dispatch an action
    pub fn dispatch(&self, action: &dyn Action) {
        // Execute middlewares
        let next = |action: &dyn Action| {
            let current_state = self.state.read().clone();
            let new_state = (self.reducer)(&current_state, action);
            
            let mut state = self.state.write();
            *state = new_state.clone();
            drop(state);

            // Notify subscribers
            let subscribers = self.subscribers.read();
            for subscriber in subscribers.iter() {
                subscriber(&new_state);
            }
        };

        if self.middlewares.is_empty() {
            next(action);
        } else {
            let current_state = self.state.read().clone();
            for middleware in &self.middlewares {
                middleware(&current_state, action, &next);
            }
        }
    }

    /// Get current state
    pub fn get_state(&self) -> S {
        self.state.read().clone()
    }

    /// Replace the entire state (use with caution)
    pub fn replace_state(&self, new_state: S) {
        let mut state = self.state.write();
        *state = new_state.clone();
        drop(state);

        // Notify subscribers
        let subscribers = self.subscribers.read();
        for subscriber in subscribers.iter() {
            subscriber(&new_state);
        }
    }
}

/// Example application state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppState {
    pub user: Option<User>,
    pub ui: UIState,
    pub data: DataState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UIState {
    pub loading: bool,
    pub error: Option<String>,
    pub theme: String,
    pub sidebar_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataState {
    pub items: Vec<String>,
    pub selected_item: Option<String>,
    pub cache: HashMap<String, String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            user: None,
            ui: UIState {
                loading: false,
                error: None,
                theme: "light".to_string(),
                sidebar_open: true,
            },
            data: DataState {
                items: Vec::new(),
                selected_item: None,
                cache: HashMap::new(),
            },
        }
    }
}

/// Example actions
#[derive(Debug, Clone)]
pub enum AppAction {
    // User actions
    LoginUser { user: User },
    LogoutUser,
    UpdateUserProfile { name: String, email: String },

    // UI actions
    SetLoading { loading: bool },
    SetError { error: Option<String> },
    SetTheme { theme: String },
    ToggleSidebar,

    // Data actions
    AddItem { item: String },
    RemoveItem { item: String },
    SelectItem { item: Option<String> },
    UpdateCache { key: String, value: String },
    ClearCache,
}

impl Action for AppAction {
    fn action_type(&self) -> &str {
        match self {
            AppAction::LoginUser { .. } => "LOGIN_USER",
            AppAction::LogoutUser => "LOGOUT_USER",
            AppAction::UpdateUserProfile { .. } => "UPDATE_USER_PROFILE",
            AppAction::SetLoading { .. } => "SET_LOADING",
            AppAction::SetError { .. } => "SET_ERROR",
            AppAction::SetTheme { .. } => "SET_THEME",
            AppAction::ToggleSidebar => "TOGGLE_SIDEBAR",
            AppAction::AddItem { .. } => "ADD_ITEM",
            AppAction::RemoveItem { .. } => "REMOVE_ITEM",
            AppAction::SelectItem { .. } => "SELECT_ITEM",
            AppAction::UpdateCache { .. } => "UPDATE_CACHE",
            AppAction::ClearCache => "CLEAR_CACHE",
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// App reducer function
pub fn app_reducer(state: &AppState, action: &dyn Action) -> AppState {
    // Try to downcast to AppAction
    let action = match action.as_any().downcast_ref::<AppAction>() {
        Some(a) => a,
        None => return state.clone(),
    };

    let mut new_state = state.clone();

    match action {
        AppAction::LoginUser { user } => {
            new_state.user = Some(user.clone());
            new_state.ui.error = None;
        }
        AppAction::LogoutUser => {
            new_state.user = None;
        }
        AppAction::UpdateUserProfile { name, email } => {
            if let Some(ref mut user) = new_state.user {
                user.name = name.clone();
                user.email = email.clone();
            }
        }
        AppAction::SetLoading { loading } => {
            new_state.ui.loading = *loading;
        }
        AppAction::SetError { error } => {
            new_state.ui.error = error.clone();
        }
        AppAction::SetTheme { theme } => {
            new_state.ui.theme = theme.clone();
        }
        AppAction::ToggleSidebar => {
            new_state.ui.sidebar_open = !new_state.ui.sidebar_open;
        }
        AppAction::AddItem { item } => {
            if !new_state.data.items.contains(item) {
                new_state.data.items.push(item.clone());
            }
        }
        AppAction::RemoveItem { item } => {
            new_state.data.items.retain(|i| i != item);
            if new_state.data.selected_item.as_ref() == Some(item) {
                new_state.data.selected_item = None;
            }
        }
        AppAction::SelectItem { item } => {
            new_state.data.selected_item = item.clone();
        }
        AppAction::UpdateCache { key, value } => {
            new_state.data.cache.insert(key.clone(), value.clone());
        }
        AppAction::ClearCache => {
            new_state.data.cache.clear();
        }
    }

    new_state
}

/// Logging middleware
pub fn logging_middleware<S: Clone + Send + Sync + std::fmt::Debug>(
    state: &S,
    action: &dyn Action,
    next: &dyn Fn(&dyn Action),
) {
    println!("Action: {:?}", action.action_type());
    println!("Previous State: {:?}", state);
    
    next(action);
    
    // Note: We can't easily get the new state here without additional infrastructure
    println!("Action dispatched");
}

/// Async middleware for handling side effects
pub fn async_middleware<S: Clone + Send + Sync>(
    _state: &S,
    action: &dyn Action,
    next: &dyn Fn(&dyn Action),
) {
    // In a real implementation, this would handle async operations
    // For now, just pass through
    next(action);
}

/// Validation middleware
pub fn validation_middleware(
    state: &AppState,
    action: &dyn Action,
    next: &dyn Fn(&dyn Action),
) {
    // Validate actions before they're processed
    let action = match action.as_any().downcast_ref::<AppAction>() {
        Some(a) => a,
        None => {
            next(action);
            return;
        }
    };

    let is_valid = match action {
        AppAction::UpdateUserProfile { name, email } => {
            !name.is_empty() && email.contains('@')
        }
        AppAction::SetTheme { theme } => {
            theme == "light" || theme == "dark"
        }
        AppAction::AddItem { item } => {
            !item.is_empty() && !state.data.items.contains(item)
        }
        _ => true,
    };

    if is_valid {
        next(action);
    } else {
        println!("Action validation failed: {:?}", action.action_type());
    }
}

/// State selector utilities
pub trait Selector<S, T> {
    fn select(&self, state: &S) -> T;
}

/// Memoized selector
pub struct MemoizedSelector<S, T> {
    selector: Arc<dyn Fn(&S) -> T + Send + Sync>,
    cache: Arc<RwLock<Option<T>>>,
}

impl<S, T: Clone + PartialEq> MemoizedSelector<S, T> {
    pub fn new(selector: Arc<dyn Fn(&S) -> T + Send + Sync>) -> Self {
        Self {
            selector,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn select(&self, state: &S) -> T {
        let result = (self.selector)(state);
        
        let mut cache = self.cache.write();
        if cache.as_ref() != Some(&result) {
            *cache = Some(result.clone());
        }
        
        result
    }
}

/// Combine multiple reducers
pub fn combine_reducers<S: Clone + Send + Sync + 'static>(
    reducers: Vec<ReducerFn<S>>,
) -> ReducerFn<S> {
    Arc::new(move |state, action| {
        let mut new_state = state.clone();
        for reducer in &reducers {
            new_state = reducer(&new_state, action);
        }
        new_state
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_creation() {
        let initial_state = AppState::default();
        let store = Store::new(initial_state.clone(), Arc::new(app_reducer));
        
        assert_eq!(store.get_state(), initial_state);
    }

    #[test]
    fn test_login_action() {
        let store = Store::new(AppState::default(), Arc::new(app_reducer));
        
        let user = User {
            id: "1".to_string(),
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            role: "admin".to_string(),
        };

        store.dispatch(&AppAction::LoginUser { user: user.clone() });
        
        let state = store.get_state();
        assert_eq!(state.user, Some(user));
    }

    #[test]
    fn test_logout_action() {
        let store = Store::new(AppState::default(), Arc::new(app_reducer));
        
        let user = User {
            id: "1".to_string(),
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            role: "admin".to_string(),
        };

        store.dispatch(&AppAction::LoginUser { user });
        store.dispatch(&AppAction::LogoutUser);
        
        let state = store.get_state();
        assert_eq!(state.user, None);
    }

    #[test]
    fn test_ui_actions() {
        let store = Store::new(AppState::default(), Arc::new(app_reducer));
        
        store.dispatch(&AppAction::SetLoading { loading: true });
        assert!(store.get_state().ui.loading);
        
        store.dispatch(&AppAction::SetTheme { theme: "dark".to_string() });
        assert_eq!(store.get_state().ui.theme, "dark");
        
        store.dispatch(&AppAction::ToggleSidebar);
        assert!(!store.get_state().ui.sidebar_open);
    }

    #[test]
    fn test_data_actions() {
        let store = Store::new(AppState::default(), Arc::new(app_reducer));
        
        store.dispatch(&AppAction::AddItem { item: "item1".to_string() });
        store.dispatch(&AppAction::AddItem { item: "item2".to_string() });
        
        let state = store.get_state();
        assert_eq!(state.data.items.len(), 2);
        assert!(state.data.items.contains(&"item1".to_string()));
        
        store.dispatch(&AppAction::SelectItem { item: Some("item1".to_string()) });
        assert_eq!(store.get_state().data.selected_item, Some("item1".to_string()));
        
        store.dispatch(&AppAction::RemoveItem { item: "item1".to_string() });
        let state = store.get_state();
        assert_eq!(state.data.items.len(), 1);
        assert_eq!(state.data.selected_item, None);
    }

    #[test]
    fn test_cache_actions() {
        let store = Store::new(AppState::default(), Arc::new(app_reducer));
        
        store.dispatch(&AppAction::UpdateCache {
            key: "key1".to_string(),
            value: "value1".to_string(),
        });
        
        let state = store.get_state();
        assert_eq!(state.data.cache.get("key1"), Some(&"value1".to_string()));
        
        store.dispatch(&AppAction::ClearCache);
        assert!(store.get_state().data.cache.is_empty());
    }

    #[test]
    fn test_subscriber() {
        let store = Store::new(AppState::default(), Arc::new(app_reducer));
        
        let state_changes = Arc::new(RwLock::new(0));
        let state_changes_clone = Arc::clone(&state_changes);
        
        store.subscribe(Arc::new(move |_state| {
            let mut count = state_changes_clone.write();
            *count += 1;
        }));
        
        store.dispatch(&AppAction::SetLoading { loading: true });
        store.dispatch(&AppAction::SetTheme { theme: "dark".to_string() });
        
        assert_eq!(*state_changes.read(), 2);
    }

    #[test]
    fn test_middleware() {
        let mut store = Store::new(AppState::default(), Arc::new(app_reducer));
        
        let action_count = Arc::new(RwLock::new(0));
        let action_count_clone = Arc::clone(&action_count);
        
        store.add_middleware(Arc::new(move |_state, _action, next| {
            let mut count = action_count_clone.write();
            *count += 1;
            next(_action);
        }));
        
        store.dispatch(&AppAction::SetLoading { loading: true });
        store.dispatch(&AppAction::SetTheme { theme: "dark".to_string() });
        
        assert_eq!(*action_count.read(), 2);
    }

    #[test]
    fn test_update_user_profile() {
        let store = Store::new(AppState::default(), Arc::new(app_reducer));
        
        let user = User {
            id: "1".to_string(),
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            role: "admin".to_string(),
        };

        store.dispatch(&AppAction::LoginUser { user });
        store.dispatch(&AppAction::UpdateUserProfile {
            name: "Alice Smith".to_string(),
            email: "alice.smith@example.com".to_string(),
        });
        
        let state = store.get_state();
        assert_eq!(state.user.as_ref().unwrap().name, "Alice Smith");
        assert_eq!(state.user.as_ref().unwrap().email, "alice.smith@example.com");
    }

    #[test]
    fn test_memoized_selector() {
        let selector = MemoizedSelector::new(Arc::new(|state: &AppState| {
            state.data.items.len()
        }));
        
        let state = AppState::default();
        let result1 = selector.select(&state);
        let result2 = selector.select(&state);
        
        assert_eq!(result1, result2);
        assert_eq!(result1, 0);
    }

    #[test]
    fn test_combine_reducers() {
        let reducer1: ReducerFn<AppState> = Arc::new(|state, _action| {
            let mut new_state = state.clone();
            new_state.ui.loading = true;
            new_state
        });

        let reducer2: ReducerFn<AppState> = Arc::new(|state, _action| {
            let mut new_state = state.clone();
            new_state.ui.theme = "dark".to_string();
            new_state
        });

        let combined = combine_reducers(vec![reducer1, reducer2]);
        let state = AppState::default();
        let new_state = combined(&state, &AppAction::SetLoading { loading: false });
        
        assert!(new_state.ui.loading);
        assert_eq!(new_state.ui.theme, "dark");
    }
}
