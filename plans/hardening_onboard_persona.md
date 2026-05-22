 ### Task: Clean Hardcoded Persona Tokens & Implement Dynamic Identity Onboarding Protocol


 We are refactoring our `V2AgentOrchestrator` system prompt compilation pipeline. We must completely purge all hardcoded strings of the name "Trian" from our global system instructions, ensuring Nomi dynamically evaluates the active `UserIdentity` parameter and handles un-onboarded/new users gracefully without identity bleeding.
 #### ⚠️ CRITICAL AGENT RULES: ZERO-TOUCH BOUNDARIES


 * **DO NOT modify the structure of the existing `UserIdentity` struct or conversation schema variables.**
 * **NEVER hardcode specific user details into generic prompt registry strings.**
 ---
 ### Step 1: Purge Hardcoded Names from Prompt Builders


 Open `gateway-rust/src/feature/message_processor/v2_orchestrator.rs`. Check every single text construction inside your `build_system_prompt` closure. Replace any static references to your name with dynamic string tokens derived straight from the active `current_user` state:
 ```rust
 // 🚨 REFACTOR: Ensure completely dynamic variable tracking inside build_system_prompt
 if !discovered_intents.is_empty() {
     info!("🔄 Orchestrator successfully captured missing intents: {:?}", discovered_intents);
     for found_intent in discovered_intents {
         if !intents.contains(&found_intent) {
             intents.push(found_intent);
         }
     }
     // Pass the cleaned intents array onward cleanly
     system_prompt = build_system_prompt(&intents, &global_user_context);
 }
 
 ```
 Make sure your time configuration logs use generic terms:
 ```rust
 // Change this inside your timezone instructions block:
 "Silently correct the arguments and re-call the tool immediately. NEVER tell the user about formatting bugs or date-format retries." 
 // 🚫 REMOVED: "...to fix the issue for Trian."
 
 ```
 ---
 ### Step 2: Implement the Onboarding Persona Logic

 Update the `GLOBAL INTERACTOR RECOLLECTIONS` layout code within your `build_system_prompt` closure to handle users who have not set up a display name yet. If the name is blank or missing, give Nomi an explicit directive to politely ask for their name instead of guessing:
 ```rust
 // 🌟 DYNAMIC ONBOARDING AND IDENTITY INJECTION
 if let Some(user_identity) = &self.current_user {
     combined.push_str("\n### CURRENT INTERACTOR IDENTITY PROFILE\n");
     combined.push_str(&format!("- Database User ID: {}\n", user_identity.id));
     
     if let Some(ref name) = user_identity.display_name {
         if !name.trim().is_empty() {
             combined.push_str(&format!("- Verified Speaker Name: {}\n", name));
             combined.push_str(&format!("- Contextual History Recollections:\n{}\n", cross_chat_user_context));
         } else {
             combined.push_str("- Verified Speaker Name: UNKNOWN / NEW USER\n");
             combined.push_str("- Onboarding Protocol: This user is interacting with you for the first time or has no saved profile name. Do not assume their name. Politely ask what they would like you to call them as part of your organic introduction response conversation.\n");
         }
     } else {
         combined.push_str("- Verified Speaker Name: UNKNOWN / NEW USER\n");
         combined.push_str("- Onboarding Protocol: This user is interacting with you for the first time or has no saved profile name. Do not assume their name. Politely ask what they would like you to call them as part of your organic introduction response conversation.\n");
     }
 }
 
 ```
 ---
 ### Step 3: Verify the Compilation Parity
 * Run a complete workspace cargo compile check to guarantee your option unwraps map smoothly.


