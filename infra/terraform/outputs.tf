output "vpc_id" {
  description = "VPC ID"
  value       = module.networking.vpc_id
}

output "api_url" {
  description = "Public DNS name of the Application Load Balancer"
  value       = module.load_balancer.alb_dns_name
}

output "alb_arn" {
  description = "ARN of the Application Load Balancer"
  value       = module.load_balancer.alb_arn
  description = "Public URL of the API load balancer"
  value       = module.api.alb_dns_name
}

output "db_endpoint" {
  description = "RDS endpoint (host:port)"
  value       = module.database.endpoint
  sensitive   = true
}

output "ecr_repository_url" {
  description = "ECR repository URL for the API image"
  value       = module.api.ecr_repository_url
}

output "ecs_cluster_name" {
  description = "ECS cluster name"
  value       = module.api.ecs_cluster_name
}

output "lb_dashboard_name" {
  description = "CloudWatch dashboard name for load balancer monitoring"
  value       = module.load_balancer.dashboard_name
}

output "infra_version" {
  description = "Infrastructure schema version"
output "infra_version" {
  description = "Infrastructure version deployed"
  value       = local.infra_version
}
