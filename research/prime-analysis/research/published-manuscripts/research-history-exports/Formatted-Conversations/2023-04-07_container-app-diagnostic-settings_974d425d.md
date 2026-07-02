# Container App Diagnostic Settings

- **Conversation ID:** `974d425d-e76d-4bea-af48-8d6fe75da4a3`
- **Date:** 2023-04-07

---

### User (2023-04-07 19:20:46)

write me a json template for a container app environment diagnostic settings pointing to an eventhub

### Assistant (2023-04-07 19:20:58)

Sure, here's a JSON template for container app environment diagnostic settings pointing to an Event Hub:

```json
{
  "name": "ContainerAppDiagnosticSettings",
  "properties": {
    "workspaceId": "<your workspace ID>",
    "eventHubAuthorizationRuleId": "<your Event Hub authorization rule ID>",
    "logs": [
      {
        "category": "ContainerLogs",
        "enabled": true,
        "retentionPolicy": {
          "days": 30,
          "enabled": true
        },
        "storageAccountId": "<your storage account ID>"
      }
    ],
    "metrics": [
      {
        "category": "AllMetrics",
        "enabled": true,
        "retentionPolicy": {
          "days": 30,
          "enabled": true
        }
      }
    ]
  }
}
```

In this template, you'll need to replace the placeholders `<your workspace ID>`, `<your Event Hub authorization rule ID>`, and `<your storage account ID>` with the actual values for your environment. 

This template enables logging of container logs and metrics for all categories, and sets a retention policy of 30 days for both logs and metrics. The logs are stored in the specified storage account, and the metrics are sent to the specified Event Hub using the specified authorization rule.

### User (2023-04-07 19:21:57)

json to direct azure monitor logging to an event hub from a container app environment

### Assistant (2023-04-07 19:22:13)

Here's a JSON template to direct Azure Monitor logging to an Event Hub from a container app environment:

```json
{
  "type": "Microsoft.ContainerInstance/containerGroups/providers/diagnosticSettings",
  "apiVersion": "2017-05-01-preview",
  "name": "containerGroupMonitoring",
  "properties": {
    "workspaceId": "<your workspace ID>",
    "eventHubAuthorizationRuleId": "<your Event Hub authorization rule ID>",
    "logs": [
      {
        "category": "ContainerLogs",
        "enabled": true,
        "retentionPolicy": {
          "enabled": true,
          "days": 7
        },
        "storageAccountId": null,
        "destination": {
          "endpointType": "EventHub",
          "properties": {
            "eventHubResourceId": "<your Event Hub resource ID>",
            "partitionKey": null
          }
        }
      }
    ]
  }
}
```

In this JSON template, you'll need to replace the placeholders `<your workspace ID>`, `<your Event Hub authorization rule ID>`, and `<your Event Hub resource ID>` with the actual values for your environment.

This template enables logging of container logs and directs the logs to the specified Event Hub using the specified authorization rule. The retention policy for the logs is set to 7 days, and the logs are not stored in a storage account. The `endpointType` property is set to "EventHub" to indicate that the logs should be sent to an Event Hub, and the `eventHubResourceId` property is set to the resource ID of the Event Hub. The `partitionKey` property is set to null, which means that the logs will be evenly distributed across all available partitions in the Event Hub.

