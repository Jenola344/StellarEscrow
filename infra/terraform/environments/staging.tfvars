environment        = "staging"
aws_region         = "us-east-1"
app_version        = "latest"

vpc_cidr           = "10.1.0.0/16"
availability_zones = ["us-east-1a", "us-east-1b"]

db_instance_class       = "db.t3.small"
db_name                 = "stellar_escrow"
db_username             = "indexer"
db_allocated_storage_gb = 20

api_image          = "stellarescrow/api:staging"
api_cpu            = 512
api_memory         = 1024
api_container_port = 3000

certificate_arn = "" # Set via TF_VAR_certificate_arn in CI
alarm_sns_arn   = "" # Set via TF_VAR_alarm_sns_arn in CI
environment         = "staging"
aws_region          = "us-east-1"
app_version         = "latest"

vpc_cidr            = "10.1.0.0/16"
availability_zones  = ["us-east-1a", "us-east-1b"]

db_instance_class        = "db.t3.small"
db_name                  = "stellar_escrow"
db_username              = "indexer"
db_allocated_storage_gb  = 20

api_image         = "stellarescrow/api:staging"
api_desired_count = 1
api_cpu           = 512
api_memory        = 1024

stellar_network     = "testnet"
stellar_horizon_url = "https://horizon-testnet.stellar.org"
stellar_contract_id = "" # Set via TF_VAR_stellar_contract_id in CI
