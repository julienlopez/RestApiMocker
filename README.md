# RestApiMocker

A small tool to mock REST APIs to better test applications

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![Build Status](https://github.com/julienlopez/RestApiMocker/actions/workflows/tests.yml/badge.svg)](https://github.com/julienlopez/RestApiMocker/actions)
[![Github Issues](https://img.shields.io/github/issues/julienlopez/RestApiMocker.svg)](http://github.com/julienlopez/RestApiMocker)
[![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)
[![Docker](https://img.shields.io/docker/pulls/julienlopez/rest-api-mocker)](https://hub.docker.com/r/julienlopez/rest-api-mocker)


## Basic Execution

to test the basic functionalites of the tool, the simplest way is just to run the docker image with the default ports

`docker run  -p 80:80 -p 9090:9090 julienlopez/rest-api-mocker:latest`

The tool is then configurable in two ways:
- via the UI (accessible on port 80 by default)
- via the REST API on /internal (the OpenAPI doc is available on /internal/docs)
