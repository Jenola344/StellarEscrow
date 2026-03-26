environment        = "development"
aws_region         = "us-east-1"
app_version        = "latest"

vpc_cidr           = "10.0.0.0/16"
availability_zones = ["us-east-1a", "us-east-1b"]

db_instance_class       = "db.t3.micro"
db_name                 = "stellar_escrow"
db_username             = "indexer"
db_allocated_storage_gb = 20

api_image          = "stellarescrow/api:latest"
api_cpu            = 256
api_memory         = 512
api_container_port = 3000

certificate_arn = "" # HTTP only in development
alarm_sns_arn   = ""
environment         = "development"
aws_region          = "us-east-1"
app_version         = "latest"

vpc_cidr            = "10.0.0.0/16"
availability_zones  = ["us-east-1a", "us-east-1b"]

db_instance_class        = "db.t3.micro"
db_name                  = "stellar_escrow"
db_username              = "indexer"
db_allocated_storage_gb  = 20

api_image         = "stellarescrow/api:latest"
api_desired_count = 1
api_cpu           = 256
api_memory        = 512

stellar_network     = "testnet"
stellar_horizon_url = "https://horizon-testnet.stellar.org"
stellar_contract_id = "CA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ"
