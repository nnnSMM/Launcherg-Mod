use thiserror::Error;

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[allow(dead_code)]
    #[error("コレクションが存在しません")]
    CollectionIsNotFound,
    #[allow(dead_code)]
    #[error("このコレクションは削除できません")]
    CollectionNotPermittedToDelete,
    #[allow(dead_code)]
    #[error("コレクションはすでに存在しています")]
    CollectionIsAlreadyExist,
    #[error("コレクションエレメントが存在しません")]
    CollectionElementIsNotFound,
    #[allow(dead_code)]
    #[error("`{0}`に有効な実行ファイルが存在しません")]
    IsNotValidPath(String),
}
