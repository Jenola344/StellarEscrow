output "vpc_id" {
  description = "VPC ID"
  value       = module.networking.vpc_id
}

output "api_url" {
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

output "infra_version" {
  description = "Infrastructure version deployed"
  value       = local.infra_version
}
