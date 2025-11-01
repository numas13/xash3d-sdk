pub type Debris = crate::env_spark::Spark;

define_export! {
    export_env_debris as export if "env-debris" {
        env_debris = env_debris::Debris,
    }
}
