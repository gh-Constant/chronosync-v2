<script setup lang="ts">
import { appUsage, appIcons } from './services/window-tracker';
</script>

<template>
  <div class="container">
    <h1>App Usage</h1>
    <table>
      <thead>
        <tr>
          <th>Icon</th>
          <th>Window</th>
          <th>Duration (s)</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="usage in appUsage" :key="usage.window">
          <td>
            <img
              v-if="appIcons[usage.window]"
              :src="appIcons[usage.window]"
              alt="icon"
              class="app-icon"
            />
            <span v-else class="icon-placeholder"></span>
          </td>
          <td>{{ usage.window }}</td>
          <td>{{ usage.duration.toFixed(2) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.container {
  padding: 20px;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th, td {
  border: 1px solid #ccc;
  padding: 8px;
  text-align: left;
}

th {
  background-color: #f4f4f4;
}

.app-icon {
  width: 24px;
  height: 24px;
  object-fit: contain;
}

.icon-placeholder {
  display: inline-block;
  width: 24px;
  height: 24px;
  background: #ddd;
  border-radius: 4px;
}
</style>
