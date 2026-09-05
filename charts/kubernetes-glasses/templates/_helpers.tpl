{{/*
Expand the name of the chart.
*/}}
{{- define "kubernetes-glasses.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "kubernetes-glasses.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Chart name and version as used by the chart label.
*/}}
{{- define "kubernetes-glasses.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "kubernetes-glasses.labels" -}}
helm.sh/chart: {{ include "kubernetes-glasses.chart" . }}
{{ include "kubernetes-glasses.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "kubernetes-glasses.selectorLabels" -}}
app.kubernetes.io/name: {{ include "kubernetes-glasses.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
The image reference, resolving tag to appVersion when unset.
*/}}
{{- define "kubernetes-glasses.image" -}}
{{- $tag := .Values.image.tag | default .Chart.AppVersion -}}
{{- printf "%s:%s" .Values.image.repository $tag -}}
{{- end }}

{{/*
Controller ServiceAccount name
*/}}
{{- define "kubernetes-glasses.controllerServiceAccountName" -}}
{{- printf "%s-controller" (include "kubernetes-glasses.fullname" .) }}
{{- end }}

{{/*
DaemonSet ServiceAccount name
*/}}
{{- define "kubernetes-glasses.daemonsetServiceAccountName" -}}
{{- printf "%s-daemonset" (include "kubernetes-glasses.fullname" .) }}
{{- end }}

{{/*
Controller service DNS name (used by frontend + daemonset)
*/}}
{{- define "kubernetes-glasses.controllerHost" -}}
{{- printf "%s-controller.%s.svc.cluster.local" (include "kubernetes-glasses.fullname" .) .Release.Namespace }}
{{- end }}
