import {Component, OnInit} from '@angular/core';
import {AuthViewModelService} from '../../services/auth-view-model.service';
import {MainComponent} from '../../../shared/components/main/main.component';
import {catchError, switchMap} from 'rxjs/operators';
import {ActivatedRoute} from '@angular/router';
import {throwError} from 'rxjs';
import {ErrorMessage} from '../../../core/models';

@Component({
  selector: 'app-verify-account',
  templateUrl: './verify-account.component.html',
  styleUrls: ['./verify-account.component.scss']
})
export class VerifyAccountComponent extends MainComponent implements OnInit {

  ready = false;

  errorMessage: ErrorMessage;

  isSuccessful = false;

  get isPageReady(): boolean {
    return this.ready;
  }

  constructor(
    private viewModel: AuthViewModelService,
    private activatedRoute: ActivatedRoute
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.activatedRoute.queryParams.pipe(
        switchMap(params => {
          return params.token ? this.viewModel.verifyAccount(params.token) : throwError(new ErrorMessage({
            code: 0,
            errorCode: 0,
            message: 'invalid_value' as any
          }));
        }),
        catchError(errorMessage => {
          if (errorMessage instanceof ErrorMessage) {
            this.errorMessage = errorMessage;
          }
          this.ready = true;
          return Promise.reject(errorMessage);
        })
      ).subscribe(_ => {
        this.ready = true;
        this.isSuccessful = true;
      })
    );
  }

}
