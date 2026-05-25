import type { ReactElement } from "react";
import LandingHero from "@/features/landing/components/LandingHero";
import LandingPopularList from "@/features/landing/components/LandingPopularList";
import LandingRecentList from "@/features/landing/components/LandingRecentList";
import LandingSearchForm from "@/features/landing/components/LandingSearchForm";
import LandingSearchHint from "@/features/landing/components/LandingSearchHint";

const DashboardPage = (): ReactElement => (
  <section className="grid grid-cols-12 gap-6">
    <LandingHero />
    <div className="col-span-12 md:col-start-3 md:col-span-8">
      <LandingSearchForm />
      <LandingSearchHint />
    </div>
    <LandingRecentList />
    <LandingPopularList />
  </section>
);

export default DashboardPage;
