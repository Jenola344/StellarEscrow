locals {
  infra_version = "1.1.0"  # bumped: load balancer module extracted + autoscaling added

  name_prefix = "stellarescrow-${var.environment}"

  env_config = {
    development = {
      db_multi_az               = false
      db_deletion_protected     = false
      api_desired_count         = 1
      enable_deletion_protection = false
      enable_stickiness         = false
      autoscaling_min           = 1
      autoscaling_max           = 2
      deletion_protection       = false
    }
    staging = {
      db_multi_az               = false
      db_deletion_protected     = false
      api_desired_count         = 1
      enable_deletion_protection = false
      enable_stickiness         = true
      autoscaling_min           = 1
      autoscaling_max           = 3
      deletion_protection       = false
    }
    production = {
      db_multi_az               = true
      db_deletion_protected     = true
      api_desired_count         = 2
      enable_deletion_protection = true
      enable_stickiness         = true
      autoscaling_min           = 2
      autoscaling_max           = 8
      deletion_protection       = true
  # Bump this when the infrastructure structure changes (independent of app version).
  infra_version = "1.0.0"

  name_prefix = "stellarescrow-${var.environment}"

  # Derived per-environment settings — mirrors config/environments/*.toml
  env_config = {
    development = {
      db_multi_az          = false
      db_deletion_protected = false
      api_desired_count    = 1
      enable_deletion_protection = false
    }
    staging = {
      db_multi_az          = false
      db_deletion_protected = false
      api_desired_count    = 1
      enable_deletion_protection = false
    }
    production = {
      db_multi_az          = true
      db_deletion_protected = true
      api_desired_count    = 2
      enable_deletion_protection = true
    }
  }

  cfg = local.env_config[var.environment]
}
