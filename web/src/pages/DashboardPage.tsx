import type { ReactElement } from "react";
import LandingHero from "@/components/LandingHero";
import LandingPopularList from "@/components/LandingPopularList";
import LandingRecentList from "@/components/LandingRecentList";
import LandingSearchForm from "@/components/LandingSearchForm";
import LandingSearchHint from "@/components/LandingSearchHint";

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
