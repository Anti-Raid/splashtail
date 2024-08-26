package db

import "reflect"

func GetCols(s any) []string {
	refType := reflect.TypeOf(s)

	var cols []string

	for _, f := range reflect.VisibleFields(refType) {
		db := f.Tag.Get("db")
		reflectOpts := f.Tag.Get("reflect")

		if db == "-" || db == "" || reflectOpts == "ignore" {
			continue
		}

		cols = append(cols, db)
	}

	return cols
}
