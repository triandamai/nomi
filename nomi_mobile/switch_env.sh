#!/bin/bash

ENV=$1

if [ "$ENV" = "dev" ]; then
    echo "Switching to DEV configuration..."
    cp config-dev.json config.json
    
    # Also update .env just in case for other tools or reference
    echo "GATEWAY_BASE_URL=http://localhost:8000/api" > .env
    echo "GATEWAY_FILE_URL=http://localhost:8000/api/files" >> .env
    echo "MQTT_HOST=b1fec516.ala.eu-central-1.emqxsl.com" >> .env
    echo "MQTT_PORT=8883" >> .env
    echo "MQTT_USERNAME=nomi-client-app" >> .env
    echo "MQTT_PASSWORD=NomiPublicPass2026" >> .env
    
    echo "Done! Active configuration is now DEV."
elif [ "$ENV" = "prod" ]; then
    echo "Switching to PROD configuration..."
    cp config-prod.json config.json
    
    # Also update .env just in case for other tools or reference
    echo "GATEWAY_BASE_URL=https://nomi-gateway.pakaiarta.id/api" > .env
    echo "GATEWAY_FILE_URL=https://nomi-gateway.pakaiarta.id/api/files" >> .env
    echo "MQTT_HOST=b1fec516.ala.eu-central-1.emqxsl.com" >> .env
    echo "MQTT_PORT=8883" >> .env
    echo "MQTT_USERNAME=nomi-client-app" >> .env
    echo "MQTT_PASSWORD=NomiPublicPass2026" >> .env
    
    echo "Done! Active configuration is now PROD."
else
    echo "Usage: ./switch_env.sh [dev|prod]"
fi
