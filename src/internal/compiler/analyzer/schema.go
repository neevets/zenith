package analyzer

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
)

type ColumnMetadata map[string]string

type TableMetadata struct {
	Columns ColumnMetadata `json:"columns"`
}

type SchemaMetadata struct {
	Tables map[string]TableMetadata `json:"tables"`
}

type SchemaManager struct {
	schema *SchemaMetadata
}

func NewSchemaManager(path string) *SchemaManager {
	sm := &SchemaManager{}
	data, err := ioutil.ReadFile(path)
	if err != nil {
		if !os.IsNotExist(err) {
			fmt.Printf("Warning: Failed to read schema file %s: %v\n", path, err)
		}
		return sm
	}

	var schema SchemaMetadata
	if err := json.Unmarshal(data, &schema); err != nil {
		fmt.Printf("Warning: Failed to parse schema file %s: %v\n", path, err)
		return sm
	}

	sm.schema = &schema
	return sm
}

func (sm *SchemaManager) ValidateQuery(table string, columns []string) []string {
	if sm.schema == nil {
		return nil
	}

	errors := []string{}
	tbl, exists := sm.schema.Tables[table]
	if !exists {
		errors = append(errors, fmt.Sprintf("Table '%s' does not exist in schema", table))
		return errors
	}

	if len(columns) == 1 && columns[0] == "*" {
		return nil
	}

	for _, col := range columns {
		if _, colExists := tbl.Columns[col]; !colExists {
			errors = append(errors, fmt.Sprintf("Column '%s' does not exist in table '%s'", col, table))
		}
	}

	return errors
}
