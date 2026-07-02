# Azure container apps monitoring

- **Conversation ID:** `68e9c0c5-8a20-8321-a959-23537c3ae20e`
- **Date:** 2025-10-10

---

### User (2025-10-10 22:29:20)

when wanting to monitor azure container apps, inside of their container app environments in azure monitor, what resource providers do I need to register on the subscription?

### Assistant (2025-10-10 22:29:21)

"when wanting to monitor azure container apps, inside of their container app environments in azure monitor, what resource providers do I need to register on the subscription?"

### Assistant (2025-10-10 22:29:37)

To monitor Azure Container Apps (within their Container App Environments) via Azure Monitor / Log Analytics, you need to make sure several resource providers are registered in the subscription. Based on Microsoft documentation and community guidance, here are the key ones:

---

## ✅ Required / Recommended Resource Providers

At minimum, you should register:

| Resource Provider Namespace | Purpose / Why Needed |
|---|---|
| `Microsoft.App` | Core resource provider for Azure Container Apps (environments, apps) citeturn0search6turn0search3turn0search11turn0search13 |
| `Microsoft.OperationalInsights` | Provides Log Analytics / workspace capabilities for storing and querying logs from the container apps. citeturn0search11turn0search0turn0search5turn0search3 |
| `Microsoft.Storage` | (Sometimes) needed especially when using diagnostics archiving or storage-based log sinks, or when dealing with jobs features. citeturn0search16turn0search18 |
| `Microsoft.Insights` | For Azure Monitor diagnostics, metric routing, and other monitoring infrastructure. (Often needed to configure diagnostic settings) citeturn0search0turn0search3turn0search5turn0search13 |

You might also (depending on your architecture) need:

- `Microsoft.ContainerRegistry` — if your container images are stored in Azure Container Registry (ACR) and you need integration, identity, or pulling permissions. (Common when container apps pull images from ACR) citeturn0search18turn0search3  
- Others such as `Microsoft.KubernetesConfiguration` / `Microsoft.ExtendedLocation` if you use hybrid / Arc / custom location / Kubernetes integration scenarios. citeturn0search15  

---

## 🔄 Sample CLI Registration Commands

Here’s how you’d register the providers via Azure CLI:

```bash
az provider register --namespace Microsoft.App
az provider register --namespace Microsoft.OperationalInsights
az provider register --namespace Microsoft.Storage
az provider register --namespace Microsoft.Insights
# Also (if needed) 
az provider register --namespace Microsoft.ContainerRegistry
```

You should wait until each provider is in *Registered* status before proceeding.  

---

If you like, I can give you a full “checklist script” (CLI + ARM/Terraform) for your subscription (or multiple subscriptions) to ensure all required providers for monitoring Container Apps are in place. Do you want me to share that?

