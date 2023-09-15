pub use openbrush::contracts::traits::psp22::*;

#[openbrush::wrapper]
pub type Psp22TokenRef = dyn PSP22;

#[openbrush::trait_definition]
pub trait Psp22Token: PSP22 {}
