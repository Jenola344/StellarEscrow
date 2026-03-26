# Secrets Manager — stores the DB connection string so ECS can inject it safely
resource "aws_secretsmanager_secret" "db_url" {
  name                    = "${local.name_prefix}/db-url"
  recovery_window_in_days = local.cfg.deletion_protection ? 7 : 0
}

resource "aws_secretsmanager_secret_version" "db_url" {
  secret_id     = aws_secretsmanager_secret.db_url.id
  secret_string = "postgres://${var.db_username}:${var.db_password}@${module.database.endpoint}/${var.db_name}"
  depends_on    = [module.database]
}

# ── Modules ──────────────────────────────────────────────────────────────────

module "networking" {
  source = "./modules/networking"

  name_prefix        = local.name_prefix
  vpc_cidr           = var.vpc_cidr
  availability_zones = var.availability_zones
}

# Load balancer is provisioned before the ECS service so its SG and TG ARNs
# are available to inject into the api module.
module "load_balancer" {
  source = "./modules/load_balancer"
module "api" {
  source = "./modules/api"

  name_prefix                = local.name_prefix
  environment                = var.environment
  vpc_id                     = module.networking.vpc_id
  public_subnet_ids          = module.networking.public_subnet_ids
  private_subnet_ids         = module.networking.private_subnet_ids
  container_port             = var.api_container_port
  health_check_path          = "/health"
  health_check_interval      = 30
  health_check_timeout       = 5
  healthy_threshold          = 2
  unhealthy_threshold        = 3
  enable_stickiness          = local.cfg.enable_stickiness
  stickiness_duration        = 86400
  enable_deletion_protection = local.cfg.enable_deletion_protection
  certificate_arn            = var.certificate_arn
  alarm_sns_arn              = var.alarm_sns_arn
  ecs_cluster_name           = module.api.ecs_cluster_name
  ecs_service_name           = module.api.ecs_service_name
  autoscaling_min_capacity   = local.cfg.autoscaling_min
  autoscaling_max_capacity   = local.cfg.autoscaling_max
  scale_out_cpu_threshold    = 70
  scale_in_cpu_threshold     = 30
  scale_out_request_threshold = 1000

  depends_on = [module.api]
}

module "api" {
  source = "./modules/api"

  name_prefix             = local.name_prefix
  environment             = var.environment
  vpc_id                  = module.networking.vpc_id
  private_subnet_ids      = module.networking.private_subnet_ids
  api_image               = var.api_image
  desired_count           = local.cfg.api_desired_count
  cpu                     = var.api_cpu
  memory                  = var.api_memory
  container_port          = var.api_container_port
  aws_region              = var.aws_region
  db_secret_arn           = aws_secretsmanager_secret.db_url.arn
  stellar_network         = var.stellar_network
  stellar_contract_id     = var.stellar_contract_id
  stellar_horizon_url     = var.stellar_horizon_url
  ecs_security_group_id   = module.load_balancer.ecs_sg_id
  api_target_group_arn    = module.load_balancer.api_target_group_arn
  api_image                  = var.api_image
  desired_count              = local.cfg.api_desired_count
  cpu                        = var.api_cpu
  memory                     = var.api_memory
  aws_region                 = var.aws_region
  db_secret_arn              = aws_secretsmanager_secret.db_url.arn
  stellar_network            = var.stellar_network
  stellar_contract_id        = var.stellar_contract_id
  stellar_horizon_url        = var.stellar_horizon_url
  enable_deletion_protection = local.cfg.enable_deletion_protection
}

module "database" {
  source = "./modules/database"

  name_prefix           = local.name_prefix
  vpc_id                = module.networking.vpc_id
  private_subnet_ids    = module.networking.private_subnet_ids
  api_security_group_id = module.load_balancer.ecs_sg_id
  api_security_group_id = module.api.api_security_group_id
  instance_class        = var.db_instance_class
  allocated_storage_gb  = var.db_allocated_storage_gb
  db_name               = var.db_name
  db_username           = var.db_username
  db_password           = var.db_password
  multi_az              = local.cfg.db_multi_az
  deletion_protection   = local.cfg.db_deletion_protected
}
