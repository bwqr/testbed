import {HttpRequest} from '@angular/common/http';
import {Subject} from 'rxjs';
import {locales} from '../locales';

export interface LocalizedEnum<T> {
  readonly key: T;
  readonly type?: string;
  readonly icon?: string | Array<string>;
  readonly localized: string;
}

export interface RetryRequest {
  req: HttpRequest<any>;
  subject: Subject<any>;
}

export class ErrorMessage {
  code: number;
  errorCode: number;
  message: LocalizedEnum<string>;

  constructor(message: ErrorMessage) {
    this.code = message.code;
    this.errorCode = message.errorCode;

    const m = message.message as any;

    let localized;
    if (m in locales.errors) {
      localized = locales.errors[m];
    } else {
      localized = m;
    }

    this.message = {
      key: m,
      localized
    };
  }
}

export class ValidationErrorMessage extends ErrorMessage {
  validationErrors: LocalizedEnum<string>[];

  constructor(message: any) {
    super(message);

    this.validationErrors = [];

    for (const e in message.validationErrors) {
      if (message.validationErrors.hasOwnProperty(e)) {
        const err = message.validationErrors[e];

        let localized;
        if (err.code in locales.errors.validationErrors) {
          localized = locales.errors.validationErrors[err.code];
        } else {
          localized = err.code;
        }

        this.validationErrors.push({
          key: err.code,
          localized
        });

        // Show first validation error in message
        if (this.validationErrors.length > 0) {
          this.message = this.validationErrors[0];
        }
      }
    }
  }
}

export interface Alert {
  text: string;
  timeout?: number;
  type: string;
  icon: string;
}

export interface Pagination<T> {
  perPage: number;
  currentPage: number;
  totalPages: number;
  totalItems: number;
  items: T[];
}

export interface PaginationParams {
  perPage: number;
  page: number;
}

export interface TokenResponse {
  token: string;
}

export interface SuccessResponse {
  message: string;
}
