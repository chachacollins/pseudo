clean:
	fd -e .c --full-path './examples' -x rm
	fd -t x --full-path './examples' -x rm
