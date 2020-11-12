#!/bin/bash
set -e

export $(cat .env | grep DATABASE_URL)
refinery migrate -e DATABASE_URL files
