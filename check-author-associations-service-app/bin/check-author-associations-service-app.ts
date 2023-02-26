#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { CheckAuthorAssociationsStack } from '../lib/check-author-associations-service-app-stack';

const app = new cdk.App();
new CheckAuthorAssociationsStack(app, 'CheckAuthorAssociationsServiceAppStack');