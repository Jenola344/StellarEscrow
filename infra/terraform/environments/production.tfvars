environment        = "production"
aws_region         = "us-east-1"
app_version        = "latest"

vpc_cidr           = "10.2.0.0/16"
availability_zones = ["us-east-1a", "us-east-1b"]

db_instance_class       = "db.t3.medium"
db_name                 = "stellar_escrow"
db_username             = "indexer"
db_allocated_storage_gb = 50

api_image          = "stellarescrow/api:production"
api_cpu            = 1024
api_memory         = 2048
api_container_port = 3000

# Inject via TF_VAR_certificate_arn — ACM cert for api.stellarescrow.io
certificate_arn = ""
# Inject via TF_VAR_alarm_sns_arn — SNS topic for PagerDuty/Slack alerts
alarm_sns_arn   = ""
environment         = "production"
aws_region          = "us-east-1"
app_version         = "latest"

vpc_cidr            = "10.2.0.0/16"
availability_zones  = ["us-east-1a", "us-east-1b"]

db_instance_class        = "db.t3.medium"
db_name                  = "stellar_escrow"
db_username              = "indexer"
db_allocated_storage_gb  = 50

api_image         = "stellarescrow/api:production"
api_desired_count = 2
api_cpu           = 1024
api_memory        = 2048

stellar_network     = "mainnet"
stellar_horizon_url = "https://horizon.stellar.org"
stellar_contract_id = "" # Set via TF_VAR_stellar_contract_id in CI
# db_password — inject via TF_VAR_db_password (never commit)
