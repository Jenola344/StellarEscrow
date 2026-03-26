variable "environment" {
  description = "Deployment environment (development | staging | production)"
  type        = string
  validation {
    condition     = contains(["development", "staging", "production"], var.environment)
    error_message = "environment must be development, staging, or production."
  }
}

variable "aws_region" {
  description = "AWS region for all resources"
  type        = string
  default     = "us-east-1"
}

variable "app_version" {
  description = "Application version tag — used for tagging and image selection"
  description = "Application version tag (e.g. 1.2.3) — used for tagging and image selection"
  type        = string
  default     = "latest"
}

# ── Networking ────────────────────────────────────────────────────────────────
# ── Networking ──────────────────────────────────────────────────────────────

variable "vpc_cidr" {
  description = "CIDR block for the VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "List of AZs to spread subnets across"
  type        = list(string)
  default     = ["us-east-1a", "us-east-1b"]
}

# ── Database ──────────────────────────────────────────────────────────────────
# ── Database ─────────────────────────────────────────────────────────────────

variable "db_instance_class" {
  description = "RDS instance class"
  type        = string
  default     = "db.t3.micro"
}

variable "db_name" {
  description = "PostgreSQL database name"
  type        = string
  default     = "stellar_escrow"
}

variable "db_username" {
  description = "PostgreSQL master username"
  type        = string
  default     = "indexer"
  sensitive   = true
}

variable "db_password" {
  description = "PostgreSQL master password — inject via TF_VAR_db_password"
  description = "PostgreSQL master password — inject via TF_VAR_db_password or secrets manager"
  type        = string
  sensitive   = true
}

variable "db_allocated_storage_gb" {
  description = "Initial RDS storage in GB"
  type        = number
  default     = 20
}

# ── API / App ─────────────────────────────────────────────────────────────────

variable "api_image" {
  description = "Docker image URI for the API service"
  description = "Docker image URI for the API service (ECR or Docker Hub)"
  type        = string
  default     = "stellarescrow/api:latest"
}

variable "api_desired_count" {
  description = "Number of API ECS tasks to run"
  type        = number
  default     = 1
}

variable "api_cpu" {
  description = "ECS task CPU units (1024 = 1 vCPU)"
  type        = number
  default     = 256
}

variable "api_memory" {
  description = "ECS task memory in MiB"
  type        = number
  default     = 512
}

variable "api_container_port" {
  description = "Port the API container listens on"
  type        = number
  default     = 3000
}

# ── Load Balancer ─────────────────────────────────────────────────────────────

variable "certificate_arn" {
  description = "ACM certificate ARN for HTTPS. Leave empty to use HTTP only (dev)."
  type        = string
  default     = ""
}

variable "alarm_sns_arn" {
  description = "SNS topic ARN for CloudWatch alarm notifications. Leave empty to disable."
  type        = string
  default     = ""
}

# ── Stellar ───────────────────────────────────────────────────────────────────

variable "stellar_network" {
  description = "Stellar network (testnet | mainnet)"
  type        = string
  default     = "testnet"
}

variable "stellar_contract_id" {
  description = "Deployed Soroban contract ID"
  type        = string
  default     = ""
}

variable "stellar_horizon_url" {
  description = "Horizon RPC endpoint"
  type        = string
  default     = "https://horizon-testnet.stellar.org"
}
