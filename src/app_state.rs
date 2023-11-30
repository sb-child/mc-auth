use std::sync::Arc;

use prisma::PrismaClient;

use crate::{prisma, settings::Settings};

type DbState = Arc<PrismaClient>;

#[derive(Clone)]
pub struct AppState {
  pub db: DbState,
  pub settings: Settings,
}
