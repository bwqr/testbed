import {Component, OnInit} from '@angular/core';
import {FormBuilder, FormGroup, Validators} from '@angular/forms';
import {MainComponent} from '../../../shared/components/main/main.component';
import {ExperimentViewModelService} from '../../services/experiment-view-model.service';
import {catchError, finalize} from 'rxjs/operators';
import {ErrorMessage} from '../../../core/models';
import {MainService} from '../../../core/services/main.service';
import {ActivatedRoute, Router} from '@angular/router';

@Component({
  selector: 'app-experiment-add',
  templateUrl: './experiment-add.component.html',
  styleUrls: ['./experiment-add.component.scss']
})
export class ExperimentAddComponent extends MainComponent implements OnInit {

  formGroup: FormGroup;

  errorMessage: ErrorMessage;

  constructor(
    private activatedRoute: ActivatedRoute,
    private viewModel: ExperimentViewModelService,
    private service: MainService,
    private formBuilder: FormBuilder,
    private router: Router,
  ) {
    super();

    this.formGroup = formBuilder.group({
      name: formBuilder.control('', [Validators.required])
    });
  }

  ngOnInit(): void {
  }

  storeExperiment(value: any): void {
    this.enterProcessingState();
    this.errorMessage = null;

    this.subs.add(
      this.viewModel.storeExperiment(value.name).pipe(
        catchError(error => {
          if (error instanceof ErrorMessage) {
            this.errorMessage = error;
          }
          return Promise.reject(error);
        }),
        finalize(() => this.leaveProcessingState())
      ).subscribe(_ => {
        this.service.alertSuccess('Experiment is created.');

        return this.router.navigate(['../experiments'], {relativeTo: this.activatedRoute});
      })
    );
  }
}
