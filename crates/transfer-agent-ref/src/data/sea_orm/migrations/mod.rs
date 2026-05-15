use sea_orm_migration::MigrationTrait;

mod m20260515_000001_transfer_processes;
mod m20260515_000002_transfer_messages;
mod m20260515_000003_transfer_identifiers;

pub fn get_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260515_000001_transfer_processes::Migration),
        Box::new(m20260515_000002_transfer_messages::Migration),
        Box::new(m20260515_000003_transfer_identifiers::Migration),
    ]
}
