import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import ParticlesBackground from "@/components/effects/ParticlesBackground";
import HeroSection from "@/components/sections/HeroSection";
import DiscordSection from "@/components/sections/DiscordSection";

export default function HomePage() {
  return (
    <>
      {/* Floating gold particles */}
      <ParticlesBackground />

      {/* Navigation */}
      <Header />

      {/* Main content */}
      <main className="relative z-10 flex-1">
        <HeroSection />
        <DiscordSection />
      </main>

      {/* Footer */}
      <Footer />
    </>
  );
}
