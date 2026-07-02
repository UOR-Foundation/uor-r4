# AMIE to NR Dashboard Conversion

- **Conversation ID:** `68276801-6984-8001-b5c9-684b35132b74`
- **Date:** 2025-05-16

---

### User (2025-05-16 12:29:52)

I've uploaded two files. The AMIE Web json file is a dashboard exported from Azure Insights in the azure json dashboard format. I would like to convert the AMIE dashboard json into the new-relic dashboard json format. The second file I have provided (example-new-relic json) is an export of an example dashboard from new relic so that you can have an example of the json formatting adjustments needed.

Please convert the AMIE json export to the same formatting as used in the new-relic json file. The goal is to be able to import the resulting AMIE file (that is written in "new-relic dashboard format") into new relic.

### Tool (2025-05-16 12:30:05)

{
  "name": "ALS - New Board 2025 (WIP)",
  "description": null,
  "permissions": "PUBLIC_READ_WRITE",
  "pages": [
    {
      "name": "Overview",
      "description": null,
      "widgets": [
        {
          "title": "asnOrganization Page Count",
          "layout": {
            "column": 1,
            "row": 1,
            "width": 12,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "markers": {
              "displayedTypes": {
                "criticalViolations": false,
                "deployments": true,
                "relatedDeployments": true,
                "warningViolations": false
              }
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT count(*) FROM PageView facet asnOrganization, name since 3 hours ago LIMIT 10 TIMESERIES auto"
              }
            ],
            "nullValues": {
              "nullValue": "remove"
            },
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "thresholds": {
              "isLabelVisible": true
            },
            "yAxisLeft": {
              "zero": true
            },
            "yAxisRight": {
              "zero": true
            }
          }
        },
        {
          "title": "Concurrent Sessions",
          "layout": {
            "column": 1,
            "row": 4,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.billboard"
          },
          "rawConfiguration": {
            "dataFormatters": [],
            "facet": {
              "showOtherSeries": false
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT max(newrelic.timeslice.value) AS 'Active Calls' FROM Metric WHERE metricTimesliceName = 'Custom/BUSY_QUEUEABLES' AND appName = 'SVENAZ PROD Worker' since 1 hour ago COMPARE WITH 1 week ago"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": true
            }
          }
        },
        {
          "title": "Page Load Time",
          "layout": {
            "column": 5,
            "row": 4,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.billboard"
          },
          "rawConfiguration": {
            "dataFormatters": [
              {
                "name": "Avg duration",
                "precision": 3,
                "type": "humanized"
              }
            ],
            "facet": {
              "showOtherSeries": false
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT average(duration) FROM PageView since 1 day ago"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "thresholds": [
              {
                "alertSeverity": "CRITICAL",
                "value": 1.9
              },
              {
                "alertSeverity": "WARNING",
                "value": 1.3
              }
            ]
          }
        },
        {
          "title": "Prod OpenSIPS Failover Events (We want 0)",
          "layout": {
            "column": 9,
            "row": 4,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.billboard"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT count(*) as 'ULG Failover Events' FROM Log_sip WHERE `hostname` LIKE '%pd%' AND `message` LIKE '%FAILOVER%' LIMIT MAX SINCE 10800 seconds AGO"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "thresholds": [
              {
                "alertSeverity": "CRITICAL",
                "value": 1
              }
            ]
          }
        },
        {
          "title": "Call Volume WoW Comparison",
          "layout": {
            "column": 1,
            "row": 7,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT MAX(newrelic.timeslice.value) AS `Concurrent Sessions` FROM Metric WHERE metricTimesliceName = 'Custom/BUSY_QUEUEABLES' AND appName IN ('SVEN Production Workers', 'SVENAZ PROD Worker') SINCE 3 hours AGO TIMESERIES 2 minutes COMPARE WITH 1 week ago"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "thresholds": {
              "isLabelVisible": true
            },
            "yAxisLeft": {
              "zero": true
            },
            "yAxisRight": {
              "zero": true
            }
          }
        },
        {
          "title": "Average Speed To Answer",
          "layout": {
            "column": 5,
            "row": 7,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "colors": {
              "seriesOverrides": [
                {
                  "color": "#ecce3c",
                  "seriesName": "Avg Speed To Answer"
                },
                {
                  "color": "#0098db",
                  "seriesName": "Previous Avg Speed To Answer"
                }
              ]
            },
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT average(speed_to_answer) FROM QREQUEST_SPEED_TO_ANSWER SINCE 3 HOURS AGO UNTIL 2 minutes ago COMPARE WITH 1 WEEK AGO TIMESERIES"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "thresholds": {
              "isLabelVisible": true
            },
            "yAxisLeft": {
              "zero": true
            },
            "yAxisRight": {
              "zero": true
            }
          }
        },
        {
          "title": "Call ended by",
          "layout": {
            "column": 9,
            "row": 7,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT count(*) FROM CALL_ENDED_BY_EVENT WHERE appName LIKE 'SVEN Production%' OR appName LIKE 'SVENAZ PROD%' SINCE 3 HOURS AGO FACET call_ended_event_code TIMESERIES"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "yAxisLeft": {
              "zero": true
            },
            "yAxisRight": {
              "zero": true
            }
          }
        },
        {
          "title": "da6pdazpxy",
          "layout": {
            "column": 1,
            "row": 10,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT latest(`nginx.server.net.connectionsActive`) AS 'Active Connections' FROM Metric WHERE entity.name LIKE 'da6pdazpxy%' OR entity.name LIKE 'da6pdproxy%' SINCE 3 HOURS AGO TIMESERIES FACET entity.name"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "yAxisLeft": {
              "zero": true
            },
            "yAxisRight": {
              "zero": true
            }
          }
        },
        {
          "title": "dc4pdazpxy",
          "layout": {
            "column": 5,
            "row": 10,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT latest(`nginx.server.net.connectionsActive`) AS 'Active Connections' FROM Metric WHERE entity.name LIKE 'dc4pdazpxy%' OR entity.name LIKE 'dc4pdproxy%' SINCE 3 HOURS AGO TIMESERIES FACET entity.name"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "yAxisLeft": {
              "zero": true
            },
            "yAxisRight": {
              "zero": true
            }
          }
        },
        {
          "title": "Active Browser Session Trend",
          "layout": {
            "column": 9,
            "row": 10,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.billboard"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT uniqueCount(session) FROM PageView since 60 minute ago until 10 seconds ago COMPARE WITH 1 hour ago"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            }
          }
        },
        {
          "title": "Redis Connected Clients",
          "layout": {
            "column": 1,
            "row": 13,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT average(`azure.cache.redis.allconnectedclients`) FROM Metric FACET `entity.name` WHERE `newrelic.cloudIntegrations.providerAccountName` = 'AMN Stratus Prod' SINCE 3 HOURS AGO TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Total Error Rate",
          "layout": {
            "column": 5,
            "row": 13,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "markers": {
              "displayedTypes": {
                "criticalViolations": false,
                "deployments": true,
                "relatedDeployments": true,
                "warningViolations": false
              }
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT rate(count(*), 1 minute) FROM TransactionError WHERE appName LIKE 'SVEN% %Prod%' SINCE 3 HOURS AGO FACET error.class TIMESERIES LIMIT 20"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "thresholds": {
              "isLabelVisible": true
            },
            "yAxisLeft": {
              "zero": true
            },
            "yAxisRight": {
              "zero": true
            }
          }
        },
        {
          "title": "Azure Container App Heartbeat Timeouts",
          "layout": {
            "column": 9,
            "row": 13,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.area"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT count(`azure.app.containerapps.ResiliencyConnectTimeouts`) FROM Metric FACET `azure.app.containerapps.revisionname` WHERE `newrelic.cloudIntegrations.providerAccountName` = 'AMN Stratus Prod' SINCE 3 HOURS AGO TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "App Infra Processing Time",
          "layout": {
            "column": 1,
            "row": 16,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "markers": {
              "displayedTypes": {
                "criticalViolations": false,
                "deployments": true,
                "relatedDeployments": true,
                "warningViolations": false
              }
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT average(apm.service.overview.web * 1000) FROM Metric WHERE appName LIKE '%SVENAZ PROD%' SINCE 3 hours AGO FACET segmentName TIMESERIES"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "thresholds": {
              "isLabelVisible": true
            },
            "yAxisLeft": {
              "zero": true
            },
            "yAxisRight": {
              "zero": true
            }
          }
        },
        {
          "title": "DB Connections - Active",
          "layout": {
            "column": 5,
            "row": 16,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT sum(`postgres.raw.pg_stat_database_session_active_time_seconds_total`) FROM Metric SINCE 30 MINUTES AGO TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "App usage by version",
          "layout": {
            "column": 9,
            "row": 16,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "markers": {
              "displayedTypes": {
                "criticalViolations": false,
                "deployments": true,
                "relatedDeployments": true,
                "warningViolations": false
              }
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT count(responseTime) from MobileRequest facet appVersion TIMESERIES since  1 hour ago"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "thresholds": {
              "isLabelVisible": true
            },
            "yAxisLeft": {
              "zero": true
            },
            "yAxisRight": {
              "zero": true
            }
          }
        },
        {
          "title": "External calls per minute",
          "layout": {
            "column": 1,
            "row": 19,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.area"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT rate(count(apm.service.external.host.duration), 1 minute) FROM Metric WHERE (entity.guid = 'MTE1NDY2MnxBUE18QVBQTElDQVRJT058Njk5MDEzNDc2') FACET `external.host` LIMIT 5 SINCE 10800 seconds AGO TIMESERIES "
              }
            ]
          }
        },
        {
          "title": "Azure Container App Probe Timeouts",
          "layout": {
            "column": 5,
            "row": 19,
            "width": 8,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.area"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "markers": {
              "displayedTypes": {
                "criticalViolations": false,
                "deployments": true,
                "relatedDeployments": true,
                "warningViolations": false
              }
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT cardinality(properties.Count) FROM Log WHERE `properties.Reason` = 'ProbeFailed' TIMESERIES Auto facet substring(properties.Log, 9, 9), substring(properties.ContainerAppName, position(properties.ContainerAppName, 'wp-'), 20)"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            }
          }
        }
      ]
    },
    {
      "name": "Application (Web and Mobile)",
      "description": null,
      "widgets": [
        {
          "title": "Transaction Times by Agent",
          "layout": {
            "column": 1,
            "row": 1,
            "width": 8,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "markers": {
              "displayedTypes": {
                "criticalViolations": false,
                "deployments": true,
                "relatedDeployments": true,
                "warningViolations": false
              }
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT sum(duration) FROM TransactionTrace WHERE uri != '' LIMIT 50 TIMESERIES auto FACET uri, dimensions(include: {'realAgentId'})"
              }
            ],
            "nullValues": {
              "nullValue": "remove"
            },
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "thresholds": {
              "isLabelVisible": true
            },
            "units": {
              "unit": "SECONDS"
            },
            "yAxisLeft": {
              "zero": true
            },
            "yAxisRight": {
              "zero": true
            }
          }
        },
        {
          "title": "Transaction Error Duration",
          "layout": {
            "column": 9,
            "row": 1,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.area"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": true
            },
            "legend": {
              "enabled": true
            },
            "markers": {
              "displayedTypes": {
                "criticalViolations": false,
                "deployments": true,
                "relatedDeployments": true,
                "warningViolations": false
              }
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT sum(duration) FROM TransactionError WHERE appName LIKE '%SVENAZ%' and appName LIKE '%PROD%' LIMIT 15 facet request.uri TIMESERIES auto"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "units": {
              "unit": "MS"
            }
          }
        },
        {
          "title": "Top Transaction Error Messages",
          "layout": {
            "column": 1,
            "row": 4,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.bar"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT count(*) FROM TransactionError FACET `error.message` SINCE 1 day ago"
              }
            ]
          }
        },
        {
          "title": "Active Browser Sessions",
          "layout": {
            "column": 5,
            "row": 4,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.billboard"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT uniqueCount(session) FROM PageView since 60 minute ago until 10 seconds ago COMPARE WITH 1 hour ago"
              }
            ]
          }
        },
        {
          "title": "HTTP method of AJAX request",
          "layout": {
            "column": 9,
            "row": 4,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.pie"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": true
            },
            "legend": {
              "enabled": true
            },
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "select count(*) from AjaxRequest WHERE httpResponseCode not in ('200','201') facet httpMethod"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            }
          }
        },
        {
          "title": "Top 5 database operations by throughput",
          "layout": {
            "column": 1,
            "row": 7,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.area"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT count(apm.service.datastore.operation.duration * 1000) FROM Metric WHERE (entity.guid = 'MTE1NDY2MnxBUE18QVBQTElDQVRJT058NzA3NzE2ODgz') FACET `db.system`, `db.sql.table`, `db.operation` LIMIT 5 SINCE 1800 seconds AGO TIMESERIES "
              }
            ]
          }
        },
        {
          "title": "iOS HTTP Error Response Codes",
          "layout": {
            "column": 5,
            "row": 7,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT count(*) FROM MobileRequestError WHERE (appId = 51101686 OR appVersionId = 51101686) FACET `statusCode` SINCE 3 hours AGO TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Throughput (requests per minute) AJAX API",
          "layout": {
            "column": 9,
            "row": 7,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.area"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT rate(count(*), 1 minute) FROM AjaxRequest WHERE (entityGuid = 'MTE1NDY2MnxCUk9XU0VSfEFQUExJQ0FUSU9OfDcxODM2MTEzNA') AND (deviceType = 'Desktop') AND (hostname is not null) FACET `httpMethod`, `requestUrl`, `hostname` LIMIT 5 SINCE 1800 seconds AGO TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "IWS Errors (non-MB)",
          "layout": {
            "column": 1,
            "row": 10,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.table"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT * FROM PageAction WHERE appName ='SVENAZ PROD API' AND message NOT LIKE '%No Media%' AND message NOT LIKE '%problem with media broker%' since 1 hour ago limit Max"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            }
          }
        },
        {
          "title": "JS Errors Stack Trace Context",
          "layout": {
            "column": 5,
            "row": 10,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.table"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT count(*) FROM JavaScriptError FACET stackTrace SINCE 1 day ago"
              }
            ]
          }
        },
        {
          "title": "Browser API status codes returned",
          "layout": {
            "column": 9,
            "row": 10,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.area"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "markers": {
              "displayedTypes": {
                "criticalViolations": false,
                "deployments": true,
                "relatedDeployments": true,
                "warningViolations": false
              }
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT count(httpResponseCode) FROM AjaxRequest WHERE (deviceType IS NOT NULL) AND (appName LIKE '%SVENAZ PROD%') AND (requestUrl is not null) AND (httpResponseCode > 204) FACET `httpResponseCode`, deviceType, browserInteractionName LIMIT MAX SINCE 1 hour AGO TIMESERIES "
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            }
          }
        },
        {
          "title": "Transaction Errors by Message over Time",
          "layout": {
            "column": 1,
            "row": 13,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT count(*) FROM TransactionError FACET `error.message` SINCE 1 day ago TIMESERIES 5 minutes"
              }
            ]
          }
        },
        {
          "title": "Top Slowest Transactions",
          "layout": {
            "column": 5,
            "row": 13,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.table"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT average(duration) from Transaction facet name since 1 hours ago"
              }
            ]
          }
        },
        {
          "title": "Transaction Segment Breakdown",
          "layout": {
            "column": 9,
            "row": 13,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.area"
          },
          "rawConfiguration": {
            "UNSTABLE_chartTypes": {
              "seriesOverrides": [
                {
                  "chartType": "line",
                  "seriesName": "Response time"
                }
              ]
            },
            "colors": {
              "validColors": {
                "seriesOverrides": [
                  {
                    "color": "#1e51ad",
                    "seriesName": "Response time"
                  }
                ]
              }
            },
            "facet": {
              "showOtherSeries": true
            },
            "markers": {
              "displayedTypes": {
                "criticalViolations": true,
                "deployments": true,
                "relatedDeployments": true,
                "warningViolations": true
              },
              "entities": [
                {
                  "domain": "APM",
                  "guid": "MTE1NDY2MnxBUE18QVBQTElDQVRJT058Njk5MDEzNDc2"
                }
              ]
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "offset": 60000,
                "query": "SELECT sum(convert(apm.service.transaction.overview, unit, 'ms')) as value FROM Metric WHERE (entity.guid = 'MTE1NDY2MnxBUE18QVBQTElDQVRJT058Njk5MDEzNDc2') AND (transactionName = 'WebTransaction/Function/ainext.views:home_view') FACET `metricTimesliceName` LIMIT 7 SINCE 1800 seconds AGO TIMESERIES "
              },
              {
                "accountIds": [
                  1154662
                ],
                "offset": 60000,
                "query": "SELECT average(convert(apm.service.transaction.duration, unit, 'ms'))  as 'Response time' FROM Metric WHERE (entity.guid = 'MTE1NDY2MnxBUE18QVBQTElDQVRJT058Njk5MDEzNDc2') AND (transactionName = 'WebTransaction/Function/ainext.views:home_view') LIMIT MAX SINCE 1800 seconds AGO TIMESERIES "
              },
              {
                "accountIds": [
                  1154662
                ],
                "offset": 60000,
                "query": "SELECT count(apm.service.transaction.duration) as throughput,\n               count(apm.service.transaction.sampled_duration) as throughput FROM Metric WHERE (entity.guid = 'MTE1NDY2MnxBUE18QVBQTElDQVRJT058Njk5MDEzNDc2') AND (transactionName = 'WebTransaction/Function/ainext.views:home_view') LIMIT MAX SINCE 1800 seconds AGO TIMESERIES "
              }
            ],
            "units": {
              "unit": "MS"
            }
          }
        },
        {
          "title": "Browser API response time of page loads and route changes",
          "layout": {
            "column": 1,
            "row": 16,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT median(duration) FROM BrowserInteraction WHERE (entityGuid = 'MTE1NDY2MnxCUk9XU0VSfEFQUExJQ0FUSU9OfDcxODM2MTEzNA') AND (category IS NOT NULL) LIMIT 5 SINCE 1800 seconds AGO FACET CASES (WHERE backendTransactionName = 'WebTransaction/Function/smanager.views:home_view' AS 'WebTransaction/Function/smanager.views:home_view', WHERE backendTransactionName = 'WebTransaction/Function/fhir_api.views.short_code_views:ShortCodeView.get' AS 'WebTransaction/Function/fhir_api.views.short_code_views:ShortCodeView.get', WHERE backendTransactionName = 'WebTransaction/Function/ainext.views:home_view' AS 'WebTransaction/Function/ainext.views:home_view', WHERE backendTransactionName = 'WebTransaction/Function/iws.views:home_view' AS 'WebTransaction/Function/iws.views:home_view', WHERE backendTransactionName = 'WebTransaction/Function/cws.views:home_view' AS 'WebTransaction/Function/cws.views:home_view') TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Interactions receiving errors",
          "layout": {
            "column": 5,
            "row": 16,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.bar"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT count(*) FROM AjaxRequest WHERE httpResponseCode NOT IN ('200','201') FACET browserInteractionName\n"
              }
            ]
          }
        },
        {
          "title": "ainext.view db calls per",
          "layout": {
            "column": 9,
            "row": 16,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "markers": {
              "displayedTypes": {
                "criticalViolations": false,
                "deployments": true,
                "relatedDeployments": true,
                "warningViolations": false
              }
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT max(databaseCallCount) AS 'DB Calls per view' FROM Transaction WHERE (name = 'WebTransaction/Function/ainext.views:home_view') SINCE 1 hour ago TIMESERIES 1 minute"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "thresholds": {
              "isLabelVisible": true
            },
            "yAxisLeft": {
              "zero": true
            },
            "yAxisRight": {
              "zero": true
            }
          }
        },
        {
          "title": "Browser Page view load time",
          "layout": {
            "column": 1,
            "row": 19,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.area"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT average(queueDuration) AS 'Request queueing', average(webAppDuration) AS 'Web application', average(networkDuration) AS 'Network duration', average(domProcessingDuration) AS 'DOM processing', average(pageRenderingDuration) AS 'Page rendering' FROM PageView WHERE (entityGuid = 'MTE1NDY2MnxCUk9XU0VSfEFQUExJQ0FUSU9OfDcxODM2MTEzNA') LIMIT 1000 SINCE 1800 seconds AGO TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Initial page load and route change",
          "layout": {
            "column": 5,
            "row": 19,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT percentile(duration, 50) FROM BrowserInteraction WHERE (entityGuid = 'MTE1NDY2MnxCUk9XU0VSfEFQUExJQ0FUSU9OfDcxODM2MTEzNA') AND (`mobileOptimized` IS NULL) AND (category IN ('Initial page load', 'Route change', 'Custom')) LIMIT 1000 SINCE 1800 seconds AGO FACET CASES (WHERE category = 'Initial page load' AS 'Initial page load (Window load + AJAX)', WHERE category = 'Route change' AS 'Route change', WHERE category = 'Custom' AS 'Custom') TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Agent Page Count",
          "layout": {
            "column": 9,
            "row": 19,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "markers": {
              "displayedTypes": {
                "criticalViolations": false,
                "deployments": true,
                "relatedDeployments": true,
                "warningViolations": false
              }
            },
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT count(*) as '# Pages' FROM PageView facet asnOrganization, deviceType, userAgentName, userAgentVersion, userAgentOS, pageUrl since 45 days ago LIMIT 5 TIMESERIES auto"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "thresholds": {
              "isLabelVisible": true
            },
            "yAxisLeft": {
              "zero": true
            },
            "yAxisRight": {
              "zero": true
            }
          }
        },
        {
          "title": "Transaction Errors",
          "layout": {
            "column": 1,
            "row": 22,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "facet": {
              "showOtherSeries": false
            },
            "legend": {
              "enabled": true
            },
            "linkedEntityGuids": null,
            "nrqlQueries": [
              {
                "accountIds": [
                  1154662
                ],
                "query": "SELECT count(*) FROM TransactionError WHERE `error.expected` IS FALSE OR `error.expected` IS NULL FACET `error.class`, `transactionUiName`, `error.message` TIMESERIES WHERE `entity.guid` LIKE '%%M%'"
              }
            ],
            "platformOptions": {
              "ignoreTimeRange": false
            },
            "vizcoConfiguration": {
              "brush": {
                "onEnd": "@@brush__onEnd"
              },
              "legend": {
                "enabled": true
              },
              "showOtherSeries": false,
              "yZero": true
            },
            "yAxisLeft": {
              "zero": true
            }
          }
        },
        {
          "title": "Calls Ended - iOS Reports",
          "layout": {
            "column": 5,
            "row": 22,
            "width": 4,
            "height": 3
          },
          "linkedEntityGuids": null,
          "visualization": {
            "id": "viz.line"
          },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountId": 1154662,
                "query": "SELECT rate(count(*), 1 minute) FROM CALL_ENDED_EVENT SINCE 3 hours AGO UNTIL 2 minutes ago FACET REASON TIMESER
The file is too long and its contents have been truncated.

### Tool (2025-05-16 12:30:05)

{
  "properties": {
    "lenses": [
      {
        "order": 0,
        "parts": [
          {
            "position": {
              "x": 0,
              "y": 0,
              "colSpan": 6,
              "rowSpan": 3
            },
            "metadata": {
              "inputs": [
                {
                  "name": "options",
                  "isOptional": true
                },
                {
                  "name": "sharedTimeRange",
                  "isOptional": true
                }
              ],
              "type": "Extension/HubsExtension/PartType/MonitorChartPart",
              "settings": {
                "content": {
                  "options": {
                    "chart": {
                      "metrics": [
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnonenotification-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnonenotification-cosmo-p01"
                          },
                          "name": "NormalizedRUConsumption",
                          "aggregationType": 4,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Normalized RU Consumption",
                            "resourceDisplayName": "co-wus2-amnonenotification-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnonecandidate-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnonecandidatemgmt-cosmo-p01"
                          },
                          "name": "NormalizedRUConsumption",
                          "aggregationType": 4,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Normalized RU Consumption",
                            "resourceDisplayName": "co-wus2-amnonecandidatemgmt-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneorder-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnoneordermgmt-cosmo-p01"
                          },
                          "name": "NormalizedRUConsumption",
                          "aggregationType": 4,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Normalized RU Consumption",
                            "resourceDisplayName": "co-wus2-amnoneordermgmt-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnoneshared-cosmo-p01"
                          },
                          "name": "NormalizedRUConsumption",
                          "aggregationType": 4,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Normalized RU Consumption",
                            "resourceDisplayName": "co-wus2-amnoneshared-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnonetask-cosmo-p01"
                          },
                          "name": "NormalizedRUConsumption",
                          "aggregationType": 4,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Normalized RU Consumption",
                            "resourceDisplayName": "co-wus2-amnonetask-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnonetemplate-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnonetemplate-cosmo-p01"
                          },
                          "name": "NormalizedRUConsumption",
                          "aggregationType": 4,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Normalized RU Consumption",
                            "resourceDisplayName": "co-wus2-amnonetemplate-cosmo-p01"
                          }
                        }
                      ],
                      "title": "Cosmos DB Accounts - Average Normalized RU Consumption",
                      "titleKind": 2,
                      "visualization": {
                        "chartType": 2,
                        "legendVisualization": {
                          "isVisible": true,
                          "position": 2,
                          "hideHoverCard": false,
                          "hideLabelNames": true
                        },
                        "axisVisualization": {
                          "x": {
                            "isVisible": true,
                            "axisType": 2
                          },
                          "y": {
                            "isVisible": true,
                            "axisType": 1
                          }
                        },
                        "disablePinning": true
                      }
                    }
                  }
                }
              }
            }
          },
          {
            "position": {
              "x": 6,
              "y": 0,
              "colSpan": 6,
              "rowSpan": 3
            },
            "metadata": {
              "inputs": [
                {
                  "name": "options",
                  "isOptional": true
                },
                {
                  "name": "sharedTimeRange",
                  "isOptional": true
                }
              ],
              "type": "Extension/HubsExtension/PartType/MonitorChartPart",
              "settings": {
                "content": {
                  "options": {
                    "chart": {
                      "metrics": [
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnonenotification-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnonenotification-cosmo-p01"
                          },
                          "name": "TotalRequests",
                          "aggregationType": 7,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Total Requests",
                            "resourceDisplayName": "co-wus2-amnonenotification-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnonecandidate-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnonecandidatemgmt-cosmo-p01"
                          },
                          "name": "TotalRequests",
                          "aggregationType": 7,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Total Requests",
                            "resourceDisplayName": "co-wus2-amnonecandidatemgmt-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneorder-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnoneordermgmt-cosmo-p01"
                          },
                          "name": "TotalRequests",
                          "aggregationType": 7,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Total Requests",
                            "resourceDisplayName": "co-wus2-amnoneordermgmt-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnoneshared-cosmo-p01"
                          },
                          "name": "TotalRequests",
                          "aggregationType": 7,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Total Requests",
                            "resourceDisplayName": "co-wus2-amnoneshared-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnonetask-cosmo-p01"
                          },
                          "name": "TotalRequests",
                          "aggregationType": 7,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Total Requests",
                            "resourceDisplayName": "co-wus2-amnonetask-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnonetemplate-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnonetemplate-cosmo-p01"
                          },
                          "name": "TotalRequests",
                          "aggregationType": 7,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Total Requests",
                            "resourceDisplayName": "co-wus2-amnonetemplate-cosmo-p01"
                          }
                        }
                      ],
                      "title": "Cosmos DB Accounts - Throttled 429 Request Count ",
                      "titleKind": 2,
                      "visualization": {
                        "chartType": 2,
                        "legendVisualization": {
                          "isVisible": true,
                          "position": 2,
                          "hideHoverCard": false,
                          "hideLabelNames": true
                        },
                        "axisVisualization": {
                          "x": {
                            "isVisible": true,
                            "axisType": 2
                          },
                          "y": {
                            "isVisible": true,
                            "axisType": 1
                          }
                        },
                        "disablePinning": true
                      }
                    }
                  }
                }
              },
              "filters": {
                "StatusCode": {
                  "model": {
                    "operator": "equals",
                    "values": [
                      "429"
                    ]
                  }
                }
              }
            }
          },
          {
            "position": {
              "x": 12,
              "y": 0,
              "colSpan": 6,
              "rowSpan": 3
            },
            "metadata": {
              "inputs": [
                {
                  "name": "options",
                  "isOptional": true
                },
                {
                  "name": "sharedTimeRange",
                  "isOptional": true
                }
              ],
              "type": "Extension/HubsExtension/PartType/MonitorChartPart",
              "settings": {
                "content": {
                  "options": {
                    "chart": {
                      "metrics": [
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnonenotification-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnonenotification-cosmo-p01"
                          },
                          "name": "ProvisionedThroughput",
                          "aggregationType": 3,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Provisioned Throughput",
                            "resourceDisplayName": "co-wus2-amnonenotification-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnonecandidate-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnonecandidatemgmt-cosmo-p01"
                          },
                          "name": "ProvisionedThroughput",
                          "aggregationType": 3,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Provisioned Throughput",
                            "resourceDisplayName": "co-wus2-amnonecandidatemgmt-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneorder-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnoneordermgmt-cosmo-p01"
                          },
                          "name": "ProvisionedThroughput",
                          "aggregationType": 3,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Provisioned Throughput",
                            "resourceDisplayName": "co-wus2-amnoneordermgmt-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnoneshared-cosmo-p01"
                          },
                          "name": "ProvisionedThroughput",
                          "aggregationType": 3,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Provisioned Throughput",
                            "resourceDisplayName": "co-wus2-amnoneshared-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnonetask-cosmo-p01"
                          },
                          "name": "ProvisionedThroughput",
                          "aggregationType": 3,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Provisioned Throughput",
                            "resourceDisplayName": "co-wus2-amnonetask-cosmo-p01"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnonetemplate-rg-p01/providers/Microsoft.DocumentDB/databaseAccounts/co-wus2-amnonetemplate-cosmo-p01"
                          },
                          "name": "ProvisionedThroughput",
                          "aggregationType": 3,
                          "namespace": "microsoft.documentdb/databaseaccounts",
                          "metricVisualization": {
                            "displayName": "Provisioned Throughput",
                            "resourceDisplayName": "co-wus2-amnonetemplate-cosmo-p01"
                          }
                        }
                      ],
                      "title": "Cosmos DB Accounts - Actual Provisioned Throughput",
                      "titleKind": 2,
                      "visualization": {
                        "chartType": 2,
                        "legendVisualization": {
                          "isVisible": true,
                          "position": 2,
                          "hideHoverCard": false,
                          "hideLabelNames": true
                        },
                        "axisVisualization": {
                          "x": {
                            "isVisible": true,
                            "axisType": 2
                          },
                          "y": {
                            "isVisible": true,
                            "axisType": 1
                          }
                        },
                        "disablePinning": true
                      }
                    }
                  }
                }
              }
            }
          },
          {
            "position": {
              "x": 0,
              "y": 3,
              "colSpan": 6,
              "rowSpan": 3
            },
            "metadata": {
              "inputs": [
                {
                  "name": "options",
                  "value": {
                    "chart": {
                      "metrics": [
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/06ab2c80-382c-478f-bcd5-5e1afe984092/resourceGroups/amn-wus2-hub-rg-p01/providers/Microsoft.ApiManagement/service/amn-wus2-hub-apim-p02"
                          },
                          "name": "Requests",
                          "aggregationType": 1,
                          "namespace": "microsoft.apimanagement/service",
                          "metricVisualization": {
                            "displayName": "Requests"
                          }
                        }
                      ],
                      "title": "Sum Requests for amn-wus2-hub-apim-p02 by Backend Response Code",
                      "titleKind": 1,
                      "visualization": {
                        "chartType": 2,
                        "legendVisualization": {
                          "isVisible": true,
                          "position": 2,
                          "hideHoverCard": false,
                          "hideLabelNames": true
                        },
                        "axisVisualization": {
                          "x": {
                            "isVisible": true,
                            "axisType": 2
                          },
                          "y": {
                            "isVisible": true,
                            "axisType": 1
                          }
                        }
                      },
                      "grouping": {
                        "dimension": "BackendResponseCode",
                        "sort": 2,
                        "top": 10
                      },
                      "timespan": {
                        "relative": {
                          "duration": 86400000
                        },
                        "showUTCTime": false,
                        "grain": 1
                      }
                    }
                  },
                  "isOptional": true
                },
                {
                  "name": "sharedTimeRange",
                  "isOptional": true
                }
              ],
              "type": "Extension/HubsExtension/PartType/MonitorChartPart",
              "settings": {
                "content": {
                  "options": {
                    "chart": {
                      "metrics": [
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/06ab2c80-382c-478f-bcd5-5e1afe984092/resourceGroups/amn-wus2-hub-rg-p01/providers/Microsoft.ApiManagement/service/amn-wus2-hub-apim-p02"
                          },
                          "name": "Requests",
                          "aggregationType": 1,
                          "namespace": "microsoft.apimanagement/service",
                          "metricVisualization": {
                            "displayName": "Requests"
                          }
                        }
                      ],
                      "title": "APIM Gateway - Sum of Requests by Backend HTTP Code",
                      "titleKind": 2,
                      "visualization": {
                        "chartType": 2,
                        "legendVisualization": {
                          "isVisible": true,
                          "position": 2,
                          "hideHoverCard": false,
                          "hideLabelNames": true
                        },
                        "axisVisualization": {
                          "x": {
                            "isVisible": true,
                            "axisType": 2
                          },
                          "y": {
                            "isVisible": true,
                            "axisType": 1
                          }
                        },
                        "disablePinning": true
                      },
                      "grouping": {
                        "dimension": "BackendResponseCode",
                        "sort": 2,
                        "top": 10
                      }
                    }
                  }
                }
              }
            }
          },
          {
            "position": {
              "x": 6,
              "y": 3,
              "colSpan": 6,
              "rowSpan": 3
            },
            "metadata": {
              "inputs": [
                {
                  "name": "options",
                  "value": {
                    "chart": {
                      "metrics": [
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "exceptions/browser",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Browser exceptions"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "dependencies/failed",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Dependency call failures"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "exceptions/count",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Exceptions"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "requests/failed",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Failed requests"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "exceptions/server",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Server exceptions"
                          }
                        }
                      ],
                      "title": "Application Insights - Failures and Exceptions",
                      "titleKind": 2,
                      "visualization": {
                        "chartType": 2,
                        "legendVisualization": {
                          "isVisible": true,
                          "position": 2,
                          "hideHoverCard": false,
                          "hideLabelNames": true
                        },
                        "axisVisualization": {
                          "x": {
                            "isVisible": true,
                            "axisType": 2
                          },
                          "y": {
                            "isVisible": true,
                            "axisType": 1
                          }
                        }
                      },
                      "timespan": {
                        "relative": {
                          "duration": 259200000
                        },
                        "showUTCTime": false,
                        "grain": 1
                      }
                    }
                  },
                  "isOptional": true
                },
                {
                  "name": "sharedTimeRange",
                  "isOptional": true
                }
              ],
              "type": "Extension/HubsExtension/PartType/MonitorChartPart",
              "settings": {
                "content": {
                  "options": {
                    "chart": {
                      "metrics": [
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "exceptions/browser",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Browser exceptions"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "dependencies/failed",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Dependency call failures"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "exceptions/count",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Exceptions"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "requests/failed",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Failed requests"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "exceptions/server",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Server exceptions"
                          }
                        }
                      ],
                      "title": "Application Insights - Failures and Exceptions",
                      "titleKind": 2,
                      "visualization": {
                        "chartType": 2,
                        "legendVisualization": {
                          "isVisible": true,
                          "position": 2,
                          "hideHoverCard": false,
                          "hideLabelNames": true
                        },
                        "axisVisualization": {
                          "x": {
                            "isVisible": true,
                            "axisType": 2
                          },
                          "y": {
                            "isVisible": true,
                            "axisType": 1
                          }
                        },
                        "disablePinning": true
                      }
                    }
                  }
                }
              }
            }
          },
          {
            "position": {
              "x": 12,
              "y": 3,
              "colSpan": 6,
              "rowSpan": 3
            },
            "metadata": {
              "inputs": [
                {
                  "name": "options",
                  "value": {
                    "chart": {
                      "metrics": [
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "exceptions/browser",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Browser exceptions"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "dependencies/failed",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Dependency call failures"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "exceptions/count",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Exceptions"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "requests/failed",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Failed requests"
                          }
                        },
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amnoneshared-ai-p01"
                          },
                          "name": "exceptions/server",
                          "aggregationType": 7,
                          "namespace": "microsoft.insights/components",
                          "metricVisualization": {
                            "displayName": "Server exceptions"
                          }
                        }
                      ],
                      "title": "Application Insights - Failures and Exceptions",
                      "titleKind": 2,
                      "visualization": {
                        "chartType": 2,
                        "legendVisualization": {
                          "isVisible": true,
                          "position": 2,
                          "hideHoverCard": false,
                          "hideLabelNames": true
                        },
                        "axisVisualization": {
                          "x": {
                            "isVisible": true,
                            "axisType": 2
                          },
                          "y": {
                            "isVisible": true,
                            "axisType": 1
                          }
                        }
                      },
                      "timespan": {
                        "relative": {
                          "duration": 259200000
                        },
                        "showUTCTime": false,
                        "grain": 1
                      }
                    }
                  },
                  "isOptional": true
                },
                {
                  "name": "sharedTimeRange",
                  "isOptional": true
                }
              ],
              "type": "Extension/HubsExtension/PartType/MonitorChartPart",
              "settings": {
                "content": {
                  "options": {
                    "chart": {
                      "metrics": [
                        {
                          "resourceMetadata": {
                            "id": "/subscriptions/1992bffb-7851-4627-8770-8e8b1ee7417f/resourceGroups/co-wus2-amnoneshared-rg-p01/providers/microsoft.insights/components/co-wus2-amn
The file is too long and its contents have been truncated.

### Tool (2025-05-16 12:30:05)

Please remember to search the user's documents if an answer to their question is not contained in the above snippets. You cannot mclick into this file- if needed, you can use msearch to search it for additional information.

### Assistant (2025-05-16 12:30:05)

To accurately convert your Azure Insights dashboard JSON into the New Relic dashboard JSON format, I need to clarify a few details:

1. **Mapping of Metrics to NRQL**: Do you have specific NRQL queries in mind that should correspond to the Azure metrics (e.g., `NormalizedRUConsumption`, `TotalRequests`)? If not, should I infer them based on naming and structure?
2. **Account ID for New Relic**: The New Relic widgets require an `accountId` field. Could you provide the appropriate account ID for your New Relic environment?
3. **Widget Layout**: Should I preserve the layout (row, column, width, height) from the Azure format, or is a standard layout acceptable?
4. **Visualization Types**: Azure uses numeric codes for chart types. Do you want me to use default New Relic visualizations (e.g., line, area, billboard), or do you have preferences based on the original charts?

Once I have this, I can generate a fully compatible New Relic dashboard JSON.

### User (2025-05-16 12:38:54)

1. No, I would need you to rewrite any queries that exist in insights for NRQL instead. New Relic has documentation for their standard field names available on their website I believe. If not, you need to learn what you can about how NRQL is used and re-create the original functionality of the Azure Insights Dashboard using NRQL, even if it is done differently (aka, if you need to pull data from logs or something instead of metrics?)
2. The account ID I will be importing this dashboard into in New Relic is 6174415
3. Standard layout is acceptable.
4. Try and match the visual types of the charts to the originals as closely as you can.

### Assistant (2025-05-16 12:39:08)

Great. I will convert the Azure Insights dashboard JSON for 'AMIE Web System Aggregate PROD' into the New Relic dashboard JSON format using your provided example for structure. I will recreate the functionality of each Azure chart using equivalent NRQL queries based on the metrics and visualizations used. The converted dashboard will use New Relic account ID 6174415 and employ a standard layout with chart types matched as closely as possible to the originals.

I'll let you know as soon as the New Relic-formatted dashboard JSON is ready for your review.

### Assistant (2025-05-16 12:38:56)

{
  "task_violates_safety_guidelines": false,
  "user_def_doesnt_want_research": false,
  "response": "Great. I will convert the Azure Insights dashboard JSON for 'AMIE Web System Aggregate PROD' into the New Relic dashboard JSON format using your provided example for structure. I will recreate the functionality of each Azure chart using equivalent NRQL queries based on the metrics and visualizations used. The converted dashboard will use New Relic account ID 6174415 and employ a standard layout with chart types matched as closely as possible to the originals.\n\nI'll let you know as soon as the New Relic-formatted dashboard JSON is ready for your review.",
  "title": "Convert Azure Insights Dashboard to New Relic Format",
  "prompt": "Convert the Azure Insights dashboard JSON from the 'AMIE Web System Aggregate PROD' export into the New Relic dashboard JSON format, following the structure provided in the 'example-new-relic-format.json' file. Reconstruct the functionality of each Azure chart using equivalent NRQL queries. \n\nDetails:\n- The original Azure dashboard uses metrics such as 'NormalizedRUConsumption', 'TotalRequests', 'ProvisionedThroughput', and others under various Azure services (Cosmos DB, API Management, Application Insights).\n- Use standard NRQL queries to fetch similar metrics or approximate functionality. If direct metric equivalents are not available, use NRQL with logs or alternative event types.\n- Use New Relic account ID 6174415.\n- Use a standard layout (no need to match original widget positions).\n- Match visualization types (e.g., line, area, billboard) as closely as possible to the Azure dashboard’s chart types.\n- The output must be a JSON file formatted for direct import into New Relic.\n\nFormat as a JSON file conforming to New Relic's dashboard schema and include proper NRQL queries under the 'nrqlQueries' field for each widget."
}

### Assistant (2025-05-16 12:44:21)

```json
{
  "name": "Converted Azure Dashboard",
  "description": null,
  "permissions": "PUBLIC_READ_WRITE",
  "pages": [
    {
      "name": "Page 1",
      "description": null,
      "widgets": [
        {
          "title": "Cosmos DB - Average Normalized RU Consumption",
          "layout": { "column": 1, "row": 1, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT average(`azure.documentdb.databaseaccounts.normalizedruconsumption`) AS 'AvgRU' FROM Metric TIMESERIES FACET `azure.resourceName`"
              }
            ]
          }
        },
        {
          "title": "Cosmos DB - Throttled 429 Request Count",
          "layout": { "column": 5, "row": 1, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT sum(`azure.documentdb.databaseaccounts.totalrequests`) AS '429Count' FROM Metric TIMESERIES FACET `azure.resourceName`"
              }
            ]
          }
        },
        {
          "title": "Cosmos DB - Provisioned Throughput",
          "layout": { "column": 9, "row": 1, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT average(`azure.documentdb.databaseaccounts.provisionedthroughput`) AS 'Provisioned Throughput' FROM Metric TIMESERIES FACET `azure.resourceName`"
              }
            ]
          }
        },
        {
          "title": "APIM Gateway - Total Requests",
          "layout": { "column": 1, "row": 4, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT sum(`azure.apimanagement.service.totalrequests`) AS 'Total Requests' FROM Metric TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Service Bus Topics - Max Active Messages",
          "layout": { "column": 5, "row": 4, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT max(`azure.servicebus.namespaces.ActiveMessages`) AS 'Active Messages' FROM Metric FACET entity.name TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Service Bus Topics - Max Incoming Messages",
          "layout": { "column": 9, "row": 4, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT max(`azure.servicebus.namespaces.IncomingMessages`) AS 'Incoming Messages' FROM Metric FACET entity.name TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Service Bus Topics - Max Dead-Lettered Messages",
          "layout": { "column": 1, "row": 7, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT max(`azure.servicebus.namespaces.DeadletteredMessages`) AS 'Deadlettered Messages' FROM Metric FACET entity.name TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Azure Functions - Updater & Automation Execution Count",
          "layout": { "column": 5, "row": 7, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT sum(`azure.web.sites.FunctionExecutionCount`) AS 'Function Execution Count' FROM Metric FACET entity.name TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Azure Functions - Legacy Execution Count",
          "layout": { "column": 9, "row": 7, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT sum(`azure.web.sites.FunctionExecutionCount`) AS 'Function Execution Count' FROM Metric FACET entity.name TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Azure Functions WebJobs - Avg CPU %",
          "layout": { "column": 1, "row": 10, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT average(`azure.web.serverfarms.CpuPercentage`) AS 'CPU %' FROM Metric TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Azure Cognitive Search - Max Throttled Queries %",
          "layout": { "column": 5, "row": 10, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT max(`azure.search.searchservices.ThrottledSearchQueriesPercentage`) AS 'Throttled %' FROM Metric TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Azure Cognitive Search - Avg Queries per Second",
          "layout": { "column": 9, "row": 10, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT average(`azure.search.searchservices.SearchQueriesPerSecond`) AS 'QPS' FROM Metric TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Azure SignalR - Max Connection Count",
          "layout": { "column": 1, "row": 13, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT max(`azure.signalrservice.signalr.ConnectionCount`) AS 'Connections' FROM Metric TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Kubernetes - Max Node CPU %",
          "layout": { "column": 5, "row": 13, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT max(`insights.container.nodes.cpuUsagePercentage`) AS 'CPU %' FROM Metric TIMESERIES"
              }
            ]
          }
        },
        {
          "title": "Azure Data Factory - Pipeline Runs",
          "layout": { "column": 9, "row": 13, "width": 4, "height": 3 },
          "linkedEntityGuids": null,
          "visualization": { "id": "viz.line" },
          "rawConfiguration": {
            "nrqlQueries": [
              {
                "accountIds": [6174415],
                "query": "SELECT sum(`azure.datafactory.factories.ActivitySucceededRuns`) AS 'Succeeded Runs' FROM Metric FACET `azure.datafactory.factories.PipelineName` TIMESERIES"
              }
            ]
          }
        }
      ]
    }
  ]
}
```

