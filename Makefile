.PHONY: help
help:     ## Show this help.
	@fgrep -h "##" $(MAKEFILE_LIST) | fgrep -v fgrep | sed -e 's/\\$$//' | sed -e 's/##//'

build: ## build the environment dockerfile
	@docker-compose build

.PHONY: pytest
pytest:   ## Run pytest test suite with coverage report
	@docker-compose run -u  $(id -u):$(id -g) open_combat bash -c "\
        set -x && redis-server \
		& cd /app/open_combat \
		&& py.test -v --cov opencombat ${PYTEST_ARGS} \
    "

.PHONY: up
up: ## Run the docker env with x windows app
	@docker run -it --rm -v=/home/${USER}/.Xauthority:/root/.Xauthority:rw \
		--net=host -e DISPLAY \
		opencombat_open_combat
