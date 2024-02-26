package perms

import "testing"

func TestHasPerm(t *testing.T) {
	if !HasPerm([]string{"global.*"}, "test") {
		t.Error("Expected true, got false")
	}
	if HasPerm([]string{"rpc.*"}, "global.*") {
		t.Error("Expected false, got true")
	}
	if !HasPerm([]string{"global.test"}, "rpc.test") {
		t.Error("Expected true, got false")
	}
	if HasPerm([]string{"global.test"}, "rpc.view_bot_queue") {
		t.Error("Expected false, got true")
	}
	if !HasPerm([]string{"global.*"}, "rpc.view_bot_queue") {
		t.Error("Expected true, got false")
	}
	if !HasPerm([]string{"rpc.*"}, "rpc.ViewBotQueue") {
		t.Error("Expected true, got false")
	}
	if HasPerm([]string{"rpc.BotClaim"}, "rpc.ViewBotQueue") {
		t.Error("Expected false, got true")
	}
	if HasPerm([]string{"apps.*"}, "rpc.ViewBotQueue") {
		t.Error("Expected false, got true")
	}
	if HasPerm([]string{"apps.*"}, "rpc.*") {
		t.Error("Expected false, got true")
	}
	if HasPerm([]string{"apps.test"}, "rpc.test") {
		t.Error("Expected false, got true")
	}
	if !HasPerm([]string{"apps.*"}, "apps.test") {
		t.Error("Expected true, got false")
	}
	if HasPerm([]string{"~apps.*"}, "apps.test") {
		t.Error("Expected false, got true")
	}
	if HasPerm([]string{"apps.*", "~apps.test"}, "apps.test") {
		t.Error("Expected false, got true")
	}
	if HasPerm([]string{"~apps.test", "apps.*"}, "apps.test") {
		t.Error("Expected false, got true")
	}
	if !HasPerm([]string{"apps.test"}, "apps.test") {
		t.Error("Expected true, got false")
	}
	if !HasPerm([]string{"apps.test", "apps.*"}, "apps.test") {
		t.Error("Expected true, got false")
	}
	if !HasPerm([]string{"~apps.test", "global.*"}, "apps.test") {
		t.Error("Expected true, got false")
	}
}

func TestResolvePerms(t *testing.T) {
	// Test for basic resolution of overrides
	expected := []string{"rpc.test"}
	result := StaffPermissions{
		UserPositions: []PartialStaffPosition{},
		PermOverrides: []string{"rpc.test"},
	}.Resolve()
	if !equal(result, expected) {
		t.Errorf("Expected %v, got %v", expected, result)
	}

	// Test for basic resolution of single position
	expected = []string{"rpc.test"}
	result = StaffPermissions{
		UserPositions: []PartialStaffPosition{
			{
				ID:    "test",
				Index: 1,
				Perms: []string{"rpc.test"},
			},
		},
		PermOverrides: []string{},
	}.Resolve()
	if !equal(result, expected) {
		t.Errorf("Expected %v, got %v", expected, result)
	}

	// Test for basic resolution of multiple positions
	expected = []string{"rpc.test2", "rpc.test"}
	result = StaffPermissions{
		UserPositions: []PartialStaffPosition{
			{
				ID:    "test",
				Index: 1,
				Perms: []string{"rpc.test"},
			},
			{
				ID:    "test2",
				Index: 2,
				Perms: []string{"rpc.test2"},
			},
		},
		PermOverrides: []string{},
	}.Resolve()
	if !equal(result, expected) {
		t.Errorf("Expected %v, got %v", expected, result)
	}

	// Test for basic resolution of multiple positions with negators
	expected = []string{"~rpc.test3", "rpc.test", "rpc.test2"}
	result = StaffPermissions{
		UserPositions: []PartialStaffPosition{
			{
				ID:    "test",
				Index: 1,
				Perms: []string{"rpc.test", "rpc.test2"},
			},
			{
				ID:    "test2",
				Index: 2,
				Perms: []string{"~rpc.test", "~rpc.test3"},
			},
		},
		PermOverrides: []string{},
	}.Resolve()
	if !equal(result, expected) {
		t.Errorf("Expected %v, got %v", expected, result)
	}

	// Same as above but testing negator ordering
	expected = []string{"~rpc.test3", "~rpc.test", "rpc.test2"}
	result = StaffPermissions{
		UserPositions: []PartialStaffPosition{
			{
				ID:    "test",
				Index: 1,
				Perms: []string{"~rpc.test", "rpc.test2"},
			},
			{
				ID:    "test2",
				Index: 2,
				Perms: []string{"~rpc.test3", "rpc.test"},
			},
		},
		PermOverrides: []string{},
	}.Resolve()
	if !equal(result, expected) {
		t.Errorf("Expected %v, got %v", expected, result)
	}

	// Now mix everything together
	expected = []string{"rpc.test2", "rpc.test3", "rpc.test"}
	result = StaffPermissions{
		UserPositions: []PartialStaffPosition{
			{
				ID:    "test",
				Index: 1,
				Perms: []string{"~rpc.test", "rpc.test2", "rpc.test3"},
			},
			{
				ID:    "test2",
				Index: 2,
				Perms: []string{"~rpc.test3", "~rpc.test2"},
			},
		},
		PermOverrides: []string{"rpc.test"},
	}.Resolve()
	if !equal(result, expected) {
		t.Errorf("Expected %v, got %v", expected, result)
	}

	// @clear
	expected = []string{"~rpc.test", "rpc.test2", "rpc.test3"}
	result = StaffPermissions{
		UserPositions: []PartialStaffPosition{
			{
				ID:    "test",
				Index: 1,
				Perms: []string{"~rpc.test", "rpc.test2"},
			},
			{
				ID:    "test",
				Index: 1,
				Perms: []string{"global.@clear", "~rpc.test", "rpc.test2"},
			},
			{
				ID:    "test2",
				Index: 2,
				Perms: []string{"~rpc.test3", "~rpc.test2"},
			},
		},
		PermOverrides: []string{"~rpc.test", "rpc.test2", "rpc.test3"},
	}.Resolve()
	if !equal(result, expected) {
		t.Errorf("Expected %v, got %v", expected, result)
	}

	// Special case of * with negators
	expected = []string{"rpc.*"}
	result = StaffPermissions{
		UserPositions: []PartialStaffPosition{
			{
				ID:    "test",
				Index: 1,
				Perms: []string{"rpc.*"},
			},
			{
				ID:    "test2",
				Index: 2,
				Perms: []string{"~rpc.test3", "~rpc.test2"},
			},
		},
		PermOverrides: []string{},
	}.Resolve()
	if !equal(result, expected) {
		t.Errorf("Expected %v, got %v", expected, result)
	}

	// Ensure special case does not apply when index is higher (2 > 1 in the below)
	expected = []string{"rpc.*", "~rpc.test3", "~rpc.test2"}
	result = StaffPermissions{
		UserPositions: []PartialStaffPosition{
			{
				ID:    "test2",
				Index: 1,
				Perms: []string{"~rpc.test3", "~rpc.test2"},
			},
			{
				ID:    "test",
				Index: 2,
				Perms: []string{"rpc.*"},
			},
		},
		PermOverrides: []string{},
	}.Resolve()
	if !equal(result, expected) {
		t.Errorf("Expected %v, got %v", expected, result)
	}

	// Some common cases
	// Ensure special case does not apply when index is higher (2 > 1 in the below)
	expected = []string{"~rpc.Claim"}
	result = StaffPermissions{
		UserPositions: []PartialStaffPosition{
			{
				ID:    "reviewer",
				Index: 1,
				Perms: []string{"rpc.Claim"},
			},
		},
		PermOverrides: []string{"~rpc.Claim"},
	}.Resolve()
	if !equal(result, expected) {
		t.Errorf("Expected %v, got %v", expected, result)
	}
}

func TestCheckPatchChanges(t *testing.T) {
	err := CheckPatchChanges([]string{"global.*"}, []string{"rpc.test"}, []string{"rpc.test", "rpc.test2"})
	if err != nil {
		t.Errorf("Expected no error, got %v", err)
	}

	err = CheckPatchChanges([]string{"rpc.*"}, []string{"global.*"}, []string{"rpc.test", "rpc.test2"})
	if err == nil {
		t.Error("Expected error, got nil")
	}

	err = CheckPatchChanges([]string{"rpc.*"}, []string{"rpc.test"}, []string{"rpc.test", "rpc.test2"})
	if err != nil {
		t.Errorf("Expected no error, got %v", err)
	}

	err = CheckPatchChanges([]string{"~rpc.test", "rpc.*"}, []string{"rpc.foobar"}, []string{"rpc.*"})
	if err == nil {
		t.Error("Expected error, got nil")
	}

	err = CheckPatchChanges([]string{"~rpc.test", "rpc.*"}, []string{"~rpc.test"}, []string{"rpc.*"})
	if err == nil {
		t.Error("Expected error, got nil")
	}

	err = CheckPatchChanges([]string{"~rpc.test", "rpc.*"}, []string{"~rpc.test"}, []string{"rpc.*", "~rpc.test", "~rpc.test2"})
	if err != nil {
		t.Errorf("Expected no error, got %v", err)
	}

	err = CheckPatchChanges([]string{"~rpc.test", "rpc.*"}, []string{"~rpc.test"}, []string{"rpc.*", "~rpc.test2", "~rpc.test2"})
	if err == nil {
		t.Error("Expected error, got nil")
	}
}

func equal(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i, v := range a {
		if v != b[i] {
			return false
		}
	}
	return true
}
