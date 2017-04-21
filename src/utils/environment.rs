use std::env;
use std::path::PathBuf;

pub struct EnvironmentUtils {}

impl EnvironmentUtils {
    pub fn sovrin_home_path() -> PathBuf {
        // TODO: FIXME: Provide better handling for the unknown home path case!!!
        let mut path = env::home_dir().unwrap_or(PathBuf::from("/sovrin"));
        path.push(".sovrin");
        path
    }

    pub fn wallet_home_path() -> PathBuf {
        let mut path = EnvironmentUtils::sovrin_home_path();
        path.push("wallet");
        path
    }

    pub fn wallet_path(wallet_name: &str) -> PathBuf {
        let mut path = EnvironmentUtils::wallet_home_path();
        path.push(wallet_name);
        path
    }

    pub fn pool_home_path() -> PathBuf {
        let mut path = EnvironmentUtils::sovrin_home_path();
        path.push("pool");
        path
    }

    pub fn pool_path(pool_name: &str) -> PathBuf {
        let mut path = EnvironmentUtils::pool_home_path();
        path.push(pool_name);
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sovrin_home_path_works() {
        let path = EnvironmentUtils::sovrin_home_path();

        assert!(path.is_absolute());
        assert!(path.has_root());
        assert!(path.to_string_lossy().contains(".sovrin"));
    }

    #[test]
    fn wallet_home_path_works() {
        let path = EnvironmentUtils::wallet_home_path();

        assert!(path.is_absolute());
        assert!(path.has_root());
        assert!(path.to_string_lossy().contains(".sovrin/wallet"));
    }

    #[test]
    fn wallet_path_works() {
        let path = EnvironmentUtils::wallet_path("wallet1");

        assert!(path.is_absolute());
        assert!(path.has_root());
        assert!(path.to_string_lossy().contains(".sovrin/wallet/wallet1"));
    }

    #[test]
    fn pool_home_path_works() {
        let path = EnvironmentUtils::pool_home_path();

        assert!(path.is_absolute());
        assert!(path.has_root());
        assert!(path.to_string_lossy().contains(".sovrin/pool"));
    }

    #[test]
    fn pool_path_works() {
        let path = EnvironmentUtils::pool_path("pool1");

        assert!(path.is_absolute());
        assert!(path.has_root());
        assert!(path.to_string_lossy().contains(".sovrin/pool/pool1"));
    }
}