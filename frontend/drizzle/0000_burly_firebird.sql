CREATE TABLE "analyses" (
	"id" varchar PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"submission_id" varchar NOT NULL,
	"engine_id" varchar NOT NULL,
	"verdict" text,
	"confidence" numeric(5, 2),
	"stake_amount" numeric(18, 8) NOT NULL,
	"details" jsonb,
	"status" text DEFAULT 'pending',
	"created_at" timestamp DEFAULT now(),
	"completed_at" timestamp
);
--> statement-breakpoint
CREATE TABLE "bounties" (
	"id" varchar PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"submission_id" varchar NOT NULL,
	"amount" numeric(18, 8) NOT NULL,
	"status" text DEFAULT 'active',
	"expires_at" timestamp,
	"created_at" timestamp DEFAULT now()
);
--> statement-breakpoint
CREATE TABLE "consensus_results" (
	"id" varchar PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"submission_id" varchar NOT NULL,
	"final_verdict" text NOT NULL,
	"confidence_score" numeric(5, 2) NOT NULL,
	"total_engines" integer NOT NULL,
	"malicious_votes" integer DEFAULT 0,
	"clean_votes" integer DEFAULT 0,
	"suspicious_votes" integer DEFAULT 0,
	"rewards_distributed" boolean DEFAULT false,
	"created_at" timestamp DEFAULT now()
);
--> statement-breakpoint
CREATE TABLE "security_engines" (
	"id" varchar PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"name" text NOT NULL,
	"type" text NOT NULL,
	"description" text,
	"accuracy" numeric(5, 2) DEFAULT '0',
	"total_analyses" integer DEFAULT 0,
	"total_staked" numeric(18, 8) DEFAULT '0',
	"status" text DEFAULT 'online',
	"owner_id" varchar,
	"created_at" timestamp DEFAULT now()
);
--> statement-breakpoint
CREATE TABLE "submissions" (
	"id" varchar PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"filename" text NOT NULL,
	"file_hash" text NOT NULL,
	"file_size" integer,
	"submission_type" text NOT NULL,
	"analysis_type" text NOT NULL,
	"bounty_amount" numeric(18, 8) NOT NULL,
	"priority" boolean DEFAULT false,
	"description" text,
	"status" text DEFAULT 'pending',
	"submitter_id" varchar,
	"created_at" timestamp DEFAULT now(),
	"completed_at" timestamp
);
--> statement-breakpoint
CREATE TABLE "users" (
	"id" varchar PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"username" text NOT NULL,
	"email" text NOT NULL,
	"password" text NOT NULL,
	"wallet_address" text,
	"reputation" numeric(10, 2) DEFAULT '0',
	"total_staked" numeric(18, 8) DEFAULT '0',
	"total_earned" numeric(18, 8) DEFAULT '0',
	"created_at" timestamp DEFAULT now(),
	CONSTRAINT "users_username_unique" UNIQUE("username"),
	CONSTRAINT "users_email_unique" UNIQUE("email")
);
--> statement-breakpoint
ALTER TABLE "analyses" ADD CONSTRAINT "analyses_submission_id_submissions_id_fk" FOREIGN KEY ("submission_id") REFERENCES "public"."submissions"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "analyses" ADD CONSTRAINT "analyses_engine_id_security_engines_id_fk" FOREIGN KEY ("engine_id") REFERENCES "public"."security_engines"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "bounties" ADD CONSTRAINT "bounties_submission_id_submissions_id_fk" FOREIGN KEY ("submission_id") REFERENCES "public"."submissions"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "consensus_results" ADD CONSTRAINT "consensus_results_submission_id_submissions_id_fk" FOREIGN KEY ("submission_id") REFERENCES "public"."submissions"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "security_engines" ADD CONSTRAINT "security_engines_owner_id_users_id_fk" FOREIGN KEY ("owner_id") REFERENCES "public"."users"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "submissions" ADD CONSTRAINT "submissions_submitter_id_users_id_fk" FOREIGN KEY ("submitter_id") REFERENCES "public"."users"("id") ON DELETE no action ON UPDATE no action;