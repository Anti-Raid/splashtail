package utils

func Must(errs ...error) {
	for _, err := range errs {
		if err != nil {
			panic(err)
		}
	}
}
