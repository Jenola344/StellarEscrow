// Infrastructure tests using Terratest.
// Run: go test -v -timeout 30m ./...
//
// These tests apply real Terraform against a dev AWS account and verify
// outputs. Set AWS credentials via environment variables before running.
package test

import (
	"testing"

	"github.com/gruntwork-io/terratest/modules/terraform"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestNetworkingModule verifies the networking module creates a VPC with the
// expected CIDR and exposes the correct outputs.
func TestNetworkingModule(t *testing.T) {
	t.Parallel()

	opts := &terraform.Options{
		TerraformDir: "../terraform/modules/networking",
		Vars: map[string]interface{}{
			"name_prefix":        "test-networking",
			"vpc_cidr":           "10.99.0.0/16",
			"availability_zones": []string{"us-east-1a", "us-east-1b"},
		},
		NoColor: true,
	}

	defer terraform.Destroy(t, opts)
	terraform.InitAndApply(t, opts)

	vpcID := terraform.Output(t, opts, "vpc_id")
	require.NotEmpty(t, vpcID, "vpc_id output must not be empty")

	publicSubnets := terraform.OutputList(t, opts, "public_subnet_ids")
	assert.Len(t, publicSubnets, 2, "expected 2 public subnets")

	privateSubnets := terraform.OutputList(t, opts, "private_subnet_ids")
	assert.Len(t, privateSubnets, 2, "expected 2 private subnets")
}

// TestDevelopmentEnvironment validates the full development stack plan
// (no apply — plan-only to keep tests fast and cost-free).
func TestDevelopmentEnvironmentPlan(t *testing.T) {
	t.Parallel()

	opts := &terraform.Options{
		TerraformDir: "../terraform",
		VarFiles:     []string{"environments/development.tfvars"},
		Vars: map[string]interface{}{
			// Inject required secrets as stubs for plan validation
			"db_password": "test-password-stub",
		},
		PlanFilePath: "/tmp/dev-plan.tfplan",
		NoColor:      true,
	}

	planOutput := terraform.InitAndPlanAndShowWithStruct(t, opts)

	// Verify key resources are planned
	assert.Contains(t, planOutput.RawPlan.PlannedValues.RootModule.Resources,
		"module.networking.aws_vpc.main",
		"VPC must be in plan",
	)
}

// TestStagingEnvironmentPlan validates the staging stack plan.
func TestStagingEnvironmentPlan(t *testing.T) {
	t.Parallel()

	opts := &terraform.Options{
		TerraformDir: "../terraform",
		VarFiles:     []string{"environments/staging.tfvars"},
		Vars: map[string]interface{}{
			"db_password":         "test-password-stub",
			"stellar_contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM",
		},
		PlanFilePath: "/tmp/staging-plan.tfplan",
		NoColor:      true,
	}

	terraform.InitAndPlan(t, opts)
}

// TestProductionEnvironmentPlan validates the production stack plan.
func TestProductionEnvironmentPlan(t *testing.T) {
	t.Parallel()

	opts := &terraform.Options{
		TerraformDir: "../terraform",
		VarFiles:     []string{"environments/production.tfvars"},
		Vars: map[string]interface{}{
			"db_password":         "test-password-stub",
			"stellar_contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM",
		},
		PlanFilePath: "/tmp/prod-plan.tfplan",
		NoColor:      true,
	}

	terraform.InitAndPlan(t, opts)
}
