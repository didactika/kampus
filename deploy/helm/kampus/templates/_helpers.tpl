# ------------------------------------------------------------------------------
#
#     _  __    _    __  __ ____  _   _ ____
#    | |/ /   / \  |  \/  |  _ \| | | / ___|
#    | ' /   / _ \ | |\/| | |_) | | | \___ \
#    | . \  / ___ \| |  | |  __/| |_| |___) |
#    |_|\_\/_/   \_\_|  |_|_|    \___/|____/
#
#    Knowledge Academic Modular Platform for Universal Study
#
# ------------------------------------------------------------------------------
#
# This file is part of Kampus - https://github.com/didactika/kampus
#
# Kampus is free software: you can redistribute it and/or modify it under the terms of the
# GNU Affero General Public License as published by the Free Software Foundation, either
# version 3 of the License, or (at your option) any later version.
#
# Kampus is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
# even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License along with Kampus.
# If not, see <https://www.gnu.org/licenses/>.
#
# @package   kampus
# @copyright 2026 Didactika
# @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
# @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
# @since     0.1.0
#
# SPDX-License-Identifier: AGPL-3.0-or-later
# ------------------------------------------------------------------------------


{{- define "kampus.name" -}}{{ .Release.Name }}{{- end }}

{{- define "kampus.labels" -}}
app.kubernetes.io/name: kampus
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "kampus.secretName" -}}
{{- if .Values.existingSecret }}{{ .Values.existingSecret }}{{ else }}{{ .Release.Name }}-secrets{{ end }}
{{- end }}

{{- define "kampus.image" -}}{{ .Values.image.repository }}:{{ .Values.image.tag }}{{- end }}

{{/* The environment of every Kampus process: the plain settings, the site and the secrets. */}}
{{- define "kampus.env" -}}
- name: KAMPUS_SITE_URL
  value: {{ .Values.siteUrl | quote }}
{{- range $name, $value := .Values.config }}
{{- if $value }}
- name: {{ $name }}
  value: {{ $value | quote }}
{{- end }}
{{- end }}
{{- end }}

{{/* The secrets of every Kampus process. */}}
{{- define "kampus.envFrom" -}}
- secretRef:
    name: {{ include "kampus.secretName" . }}
{{- end }}
