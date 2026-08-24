<script setup>
import DocumentationGuide from './components/docs/DocumentationGuide.vue'
import FeatureGrid from './components/landing/FeatureGrid.vue'
import HeroSection from './components/landing/HeroSection.vue'
import SiteFooter from './components/site/SiteFooter.vue'
import SiteHeader from './components/site/SiteHeader.vue'
</script>

<template>
  <SiteHeader />
  <main id="top">
    <HeroSection />
    <FeatureGrid />
    <DocumentationGuide />
    <SiteFooter />
  </main>
</template>
