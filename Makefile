.PHONY: help
help:     ## Show this help.
	@fgrep -h "##" $(MAKEFILE_LIST) | fgrep -v fgrep | sed -e 's/\\$$//' | sed -e 's/##//'

.PHONY: pytest
pytest:   ## Run pytest test suite with coverage report
	@docker-compose run -u  $(id -u):$(id -g) open_combat bash -c "\
        set -x && redis-server \
		& cd /app/open_combat \
		&& py.test -v --cov opencombat ${PYTEST_ARGS} \
    "

	

