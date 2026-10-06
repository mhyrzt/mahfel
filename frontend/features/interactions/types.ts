import type { components } from "@/lib/api/generated/schema";

export type InteractionsSnapshot = components["schemas"]["InteractionsResponse"];
export type Activity = components["schemas"]["ActivityResponse"];
export type ActivityDefinition = components["schemas"]["ActivityDefinitionRequest"];
export type InteractionAction = components["schemas"]["InteractionAction"];
export type RaisedHand = components["schemas"]["HandResponse"];
export type VotersPage = components["schemas"]["VotersResponse"];
export type SendInteraction = (action: InteractionAction) => Promise<boolean>;
