import {Component, ElementRef, OnInit, ViewChild} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {ExperimentViewModelService} from '../../services/experiment-view-model.service';
import {Experiment, JobUpdate, SlimJob, SlimRunner} from '../../models';
import {ActivatedRoute} from '@angular/router';
import {filter, finalize, map, switchMap} from 'rxjs/operators';
import CodeMirror from 'codemirror';
import * as python from 'codemirror/mode/python/python.js';
import {MainService} from '../../../core/services/main.service';
import {FormBuilder, FormGroup, Validators} from '@angular/forms';
import {combineLatest} from 'rxjs';
import {Pagination} from '../../../core/models';
import {WebSocketService} from '../../../core/services/web-socket.service';
import {NotificationKind} from '../../../core/websocket/models';

// This expression is required since we want python to be imported and included in output of webpack
// tslint:disable-next-line:no-unused-expression
python;

@Component({
  selector: 'app-experiment',
  templateUrl: './experiment.component.html',
  styleUrls: ['./experiment.component.scss']
})
export class ExperimentComponent extends MainComponent implements OnInit {

  experiment: Experiment;

  runners: SlimRunner[];

  jobs: Pagination<[SlimJob, SlimRunner]>;

  @ViewChild('code') code: ElementRef;

  codeMirror: any;

  formGroup: FormGroup;

  get isPageReady(): boolean {
    return !!this.experiment && !!this.runners && !!this.jobs;
  }

  constructor(
    private viewModel: ExperimentViewModelService,
    private service: MainService,
    private activatedRoute: ActivatedRoute,
    private formBuilder: FormBuilder,
    private webSocketService: WebSocketService
  ) {
    super();

    this.formGroup = formBuilder.group({
      runnerId: formBuilder.control('', [Validators.required]),
    });

    this.webSocketService.listenNotifications().pipe(
      filter(notification => notification.message.kind === NotificationKind.JobUpdate),
      map(notification => notification.message.data as JobUpdate)
    ).subscribe(notification => {
      const index = this.jobs.items.findIndex(jr => jr[0].id === notification.jobId);

      if (index > -1) {
        this.jobs.items[index][0].status = notification.status;
      }
    });
  }

  ngOnInit(): void {
    this.subs.add(
      this.viewModel.runners().subscribe(runners => this.runners = runners)
    );

    this.subs.add(
      this.activatedRoute.params.pipe(
        switchMap((params) => combineLatest([
          this.viewModel.experiment(params.id),
          this.viewModel.experimentJobs(params.id)
        ]))
      ).subscribe(([experiment, jobs]) => {
        this.experiment = experiment;
        this.jobs = jobs;

        // experiment.code is html encoded, we need to decode it
        const el = document.createElement('div');
        el.innerHTML = this.experiment.code;
        const renderedCode = el.textContent;

        this.codeMirror = CodeMirror((elt) => {
          // remove all of the children https://stackoverflow.com/questions/3955229/remove-all-child-elements-of-a-dom-node-in-javascript
          this.code.nativeElement.textContent = '';
          // append our codemirror
          this.code.nativeElement.appendChild(elt);
        }, {
          value: renderedCode,
          mode: 'python',
          lineNumbers: true,
          lineWiseCopyCut: true,
          indentUnit: 4
        });
      })
    );
  }

  saveExperiment(): void {
    this.enterProcessingState();

    this.subs.add(
      this.viewModel.updateExperiment(this.experiment.id, this.codeMirror.getValue()).pipe(
        finalize(() => this.leaveProcessingState())
      )
        .subscribe(_ => this.service.alertSuccess('Experiment is updated.'))
    );
  }

  runExperiment(value: { runnerId: number }): void {
    this.enterProcessingState();
    const runner = this.runners.find(r => r.id === value.runnerId);

    this.subs.add(
      this.viewModel.runExperiment(this.experiment.id, value.runnerId).pipe(
        finalize(() => this.leaveProcessingState())
      )
        .subscribe(job => {
          this.jobs.items.unshift([job, runner]);
          this.service.alertSuccess('Experiment is queued to run');
        })
    );
  }
}
