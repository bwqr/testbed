import {Component, OnInit} from '@angular/core';
import {ErrorMessage} from '../../../core/models';
import {FormBuilder, FormGroup, ValidationErrors, ValidatorFn, Validators} from '@angular/forms';
import {AuthViewModelService} from '../../services/auth-view-model.service';
import {ActivatedRoute} from '@angular/router';
import {MainComponent} from '../../../shared/components/main/main.component';
import {catchError, finalize} from 'rxjs/operators';
import {locales} from '../../../locales';

@Component({
  selector: 'app-reset-password',
  templateUrl: './reset-password.component.html',
  styleUrls: ['./reset-password.component.scss']
})
export class ResetPasswordComponent extends MainComponent implements OnInit {

  token: string;

  invalidToken = false;

  isSuccessful = false;

  errorMessage: ErrorMessage;

  formGroup: FormGroup;

  get isPageReady(): boolean {
    return !!this.token;
  }

  constructor(
    private viewModel: AuthViewModelService,
    private activatedRoute: ActivatedRoute,
    private formBuilder: FormBuilder,
  ) {
    super();

    this.formGroup = formBuilder.group({
      password: formBuilder.control('', [Validators.required]),
      passwordConfirm: formBuilder.control('', [Validators.required, passwordMatchValidator])
    });
  }

  ngOnInit(): void {
    this.subs.add(
      this.activatedRoute.queryParams.subscribe(params => {
        this.invalidToken = !params.token;
        this.token = params.token;
      })
    );
  }

  resetPassword(value: any): void {
    this.errorMessage = null;
    this.enterProcessingState();

    this.subs.add(
      this.viewModel.resetPassword(this.token, value.password).pipe(
        finalize(() => this.leaveProcessingState()),
        catchError(error => {
          if (error instanceof ErrorMessage) {
            this.errorMessage = error;

            // Invalid Value
            if (error.errorCode === 108) {
              this.errorMessage.message = {
                key: error.message.key,
                localized: locales.auth.errors.invalidOrExpiredToken
              };
            }
          }
          return Promise.reject(error);
        })
      )
        .subscribe(_ => this.isSuccessful = true)
    );
  }
}

const passwordMatchValidator: ValidatorFn = (control: FormGroup): ValidationErrors | null => {
  if (!control.parent) {
    return null;
  }

  const password = control.parent.get('password');

  return control.value !== password.value ? {passwordMismatch: true} : null;
};

