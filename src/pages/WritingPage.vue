<script setup lang="ts">
import { useI18n } from "@/lib/i18n";

const { t } = useI18n();
</script>

<template>
  <v-container fluid class="pa-6">
    <v-row>
      <v-col cols="12">
        <h1>{{ t('navigation.writing') }}</h1>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12">
        <v-card>
          <v-card-text>
            <p>Writing feature coming soon...</p>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>
