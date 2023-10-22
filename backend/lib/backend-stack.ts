import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { Runtime, Function, Code } from 'aws-cdk-lib/aws-lambda';
import * as sns from 'aws-cdk-lib/aws-sns';
import { Stack, RemovalPolicy } from 'aws-cdk-lib';
import { Rule, Schedule } from 'aws-cdk-lib/aws-events';
import { LambdaFunction } from 'aws-cdk-lib/aws-events-targets';
import { RetentionDays } from 'aws-cdk-lib/aws-logs';
import { LambdaIntegration, RestApi, Cors } from 'aws-cdk-lib/aws-apigateway';
import { AttributeType, Table } from 'aws-cdk-lib/aws-dynamodb';
import { Duration } from 'aws-cdk-lib';
import * as dotenv from 'dotenv';
import path = require('path');

export class BackendStack extends Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);
    dotenv.config({ path: path.resolve(__dirname, '.env') });
    const plaidApiKey = process.env.PLAID_API_KEY || 'NO API KEY SET';
    const plaidClientKey = process.env.PLAID_CLIENT_ID || 'NO CLIENT ID SET';
    const tableName = 'PlaidAccessKeys';
    const TOPIC_NAME = "WeeklyReportTopic";

    // Setup our dynamo db table
    const dynamoTable = new Table(this, tableName, {
      partitionKey: {
        name: 'uuid',
        type: AttributeType.STRING
      },
      readCapacity: 1,
      writeCapacity: 1,
      tableName: tableName,
      removalPolicy: RemovalPolicy.RETAIN,
    });

    /**
     * Get Link Lambda needs API Gateway integration
     */
    const getLink = new Function(this, 'getLink', {
      description: "Get Temporary Access Token Link from Plaid",
      code: Code.fromAsset('lib/lambdas/getLink/target/x86_64-unknown-linux-musl/release/lambda'),
      runtime: Runtime.PROVIDED_AL2,
      handler: 'not.required',
      timeout: Duration.minutes(5),
      environment: {
        RUST_BACKTRACE: '1',
        PLAID_API_KEY: plaidApiKey,
        PLAID_CLIENT_ID: plaidClientKey
      },
      logRetention: RetentionDays.ONE_WEEK,
    });

    /**
     * Create Access Token needs API Gateway and Dynamo DB Permission
     */
    const createAccessToken = new Function(this, 'createAccessToken', {
      description: "Add recipes worker",
      code: Code.fromAsset('lib/lambdas/createAccessToken/target/x86_64-unknown-linux-musl/release/lambda'),
      runtime: Runtime.PROVIDED_AL2,
      handler: 'not.required',
      timeout: Duration.minutes(5),
      environment: {
        RUST_BACKTRACE: '1',
        TABLE_NAME: tableName,
        PLAID_API_KEY: plaidApiKey,
        PLAID_CLIENT_ID: plaidClientKey
      },
      logRetention: RetentionDays.ONE_WEEK,
    });

    /**
     * Send Weekly Report needs Event Bridge and SNS Topic and Dynamo DB Read Access
     */
    const sendWeeklyReport = new Function(this, 'sendWeeklyReport', {
      description: "Add recipes worker",
      code: Code.fromAsset('lib/lambdas/sendWeeklyReport/target/x86_64-unknown-linux-musl/release/lambda'),
      runtime: Runtime.PROVIDED_AL2,
      handler: 'not.required',
      timeout: Duration.minutes(5),
      environment: {
        RUST_BACKTRACE: '1',
        TABLE_NAME: tableName,
        PLAID_API_KEY: plaidApiKey,
        PLAID_CLIENT_ID: plaidClientKey,
        TOPIC: TOPIC_NAME
      },
      logRetention: RetentionDays.ONE_WEEK,
    });

    const reportTopic = new sns.Topic(this, TOPIC_NAME);
    reportTopic.grantPublish(sendWeeklyReport);

    dynamoTable.grantFullAccess(createAccessToken);
    dynamoTable.grantFullAccess(sendWeeklyReport);

    // Create an API Gateway resource for each of the CRUD operations
    const link = new RestApi(this, 'GetLink', {
      restApiName: 'Get Link API',
      defaultCorsPreflightOptions: {
        allowOrigins: Cors.ALL_ORIGINS,
        allowMethods: Cors.ALL_METHODS,
        allowHeaders: Cors.DEFAULT_HEADERS,
      }
    });

    const accToken = new RestApi(this, 'CreateAccessToken', {
      restApiName: "Create Access Token API",
      defaultCorsPreflightOptions: {
        allowOrigins: Cors.ALL_ORIGINS,
        allowMethods: Cors.ALL_METHODS,
        allowHeaders: Cors.DEFAULT_HEADERS
      }
    });

    // Integrate lambda functions with an API gateway
    const linkApi = new LambdaIntegration(getLink);
    const accTokenApi = new LambdaIntegration(createAccessToken);

    const links = link.root.addResource('api');
    links.addMethod('GET', linkApi);

    const tokens = accToken.root.addResource('api');
    tokens.addMethod('POST', accTokenApi);

    const eventRule = new Rule(this, 'scheduleRule', {
      schedule: Schedule.expression('rate(7 days)'),
    });
    eventRule.addTarget(new LambdaFunction(sendWeeklyReport));
  }
}
