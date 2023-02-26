import { Runtime, Function, Code } from 'aws-cdk-lib/aws-lambda';
import { App, Stack } from 'aws-cdk-lib';
import { Rule, Schedule } from 'aws-cdk-lib/aws-events';
import { LambdaFunction } from 'aws-cdk-lib/aws-events-targets';
import { RetentionDays } from 'aws-cdk-lib/aws-logs';

export class CheckAuthorAssociationsStack extends Stack {
  constructor(app: App, id: string) {
    super(app, id);
    // Create a Lambda function for each of the CRUD operations
    // Lambdas needed:
    // 1. Add RARE book
    // 2. Scheduled Lambda that takes all RARE books, checks Deso for current ownership, pays author, and deletes item
    const checkAssociations = new Function(this, 'checkAssociations', {
      description: "Check All Author Associations to Ensure Subscription is Valid",
      code: Code.fromAsset('lib/lambdas/checkAssociations/target/x86_64-unknown-linux-musl/release/lambda'),
      runtime: Runtime.PROVIDED_AL2,
      handler: 'not.required',
      environment: {
        RUST_BACKTRACE: '1',
        ASSOCIATION_TYPE: 'Spatium Author',
      },
      logRetention: RetentionDays.ONE_WEEK
    });


    // Create schedule for checkRareBooks lambda
    const eventRule = new Rule(this, 'scheduleRule', {
      schedule: Schedule.expression('rate(1 day)'),
    });
    eventRule.addTarget(new LambdaFunction(checkAssociations));
  }
}

const app = new App();
new CheckAuthorAssociationsStack(app, 'CheckAuthorAssociationsStack');
app.synth();
