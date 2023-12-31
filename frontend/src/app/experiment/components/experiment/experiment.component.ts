import {Component, ElementRef, OnInit, ViewChild} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {ExperimentViewModelService} from '../../services/experiment-view-model.service';
import {Experiment, JobUpdate, SlimJob, SlimController} from '../../models';
import {ActivatedRoute} from '@angular/router';
import {catchError, filter, finalize, map, switchMap} from 'rxjs/operators';
import {MainService} from '../../../core/services/main.service';
import {FormBuilder, FormGroup, Validators} from '@angular/forms';
import {combineLatest} from 'rxjs';
import {ErrorMessage, Pagination} from '../../../core/models';
import {WebSocketService} from '../../../core/services/web-socket.service';
import {NotificationKind} from '../../../core/websocket/models';
import {basicSetup, EditorState, EditorView} from '@codemirror/basic-setup';
import {python} from '@codemirror/lang-python';
import {keymap} from '@codemirror/view';
import {indentLess, indentMore} from '@codemirror/commands';
import {Transaction} from '@codemirror/state';
import {indentUnit} from '@codemirror/language';


@Component({
  selector: 'app-experiment',
  templateUrl: './experiment.component.html',
  styleUrls: ['./experiment.component.scss']
})
export class ExperimentComponent extends MainComponent implements OnInit {

  experiment: Experiment;

  controllers: SlimController[];

  jobs: Pagination<[SlimJob, SlimController]>;

  @ViewChild('code') code: ElementRef;

  editor: EditorView;

  formGroup: FormGroup;

  get isPageReady(): boolean {
    return !!this.experiment && !!this.controllers && !!this.jobs;
  }

  constructor(
    private viewModel: ExperimentViewModelService,
    private service: MainService,
    private activatedRoute: ActivatedRoute,
    private formBuilder: FormBuilder,
    private webSocketService: WebSocketService
  ) {
    super();

    this.formGroup = this.formBuilder.group({
      controllerId: formBuilder.control('', [Validators.required]),
    });

    this.subs.add(this.webSocketService.listenNotifications().pipe(
      filter(notification => notification.message.kind === NotificationKind.JobUpdate),
      map(notification => notification.message.data as JobUpdate)
    ).subscribe(notification => {
      const index = this.jobs.items.findIndex(jr => jr[0].id === notification.jobId);

      if (index > -1) {
        this.jobs.items[index][0].status = notification.status;
      }
    }));
  }

  ngOnInit(): void {
    this.subs.add(
      this.viewModel.controllers().subscribe(controllers => this.controllers = controllers)
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
        const el = document.createElement('textarea');
        el.innerHTML = this.experiment.code;
        const renderedCode = el.textContent;

        // experiment.code is html encoded, we need to decode it
        this.editor = new EditorView({
          state: EditorState.create({
            doc: renderedCode,
            extensions: [
              basicSetup,
              python(),
              indentUnit.of(' '.repeat(4)),
              keymap.of([
                {
                  key: 'Tab',
                  run: ({state, dispatch}) => {
                    if (state.selection.ranges.some(r => !r.empty)) {
                      return indentMore({state, dispatch});
                    }
                    dispatch(state.update(state.replaceSelection(' '.repeat(4)), {
                      scrollIntoView: true,
                      annotations: Transaction.userEvent.of('input')
                    }));
                    return true;
                  }, shift: indentLess
                },
                {
                  key: 'Mod-s',
                  run: ({state, dispatch}) => {
                    if (!this.isInProcessingState) {
                      this.saveExperiment();
                    }
                    return true;
                  },
                  preventDefault: true
                },
              ])
            ]
          }),
          parent: this.code.nativeElement
        });
      })
    );
  }

  saveExperiment(): void {
    this.enterProcessingState();

    // server expects indentations to be 4 spaces, not tabs. We replace any tabs with four spaces.
    const code = this.editor.state.doc.toString().replace('\t', '    ');

    this.subs.add(
      this.viewModel.updateExperiment(this.experiment.id, code).pipe(
        finalize(() => this.leaveProcessingState())
      )
        .subscribe(_ => this.service.alertSuccess('Experiment is updated.'))
    );
  }

  runExperiment(value: { controllerId: number }): void {
    this.enterProcessingState();
    const controller = this.controllers.find(r => r.id === value.controllerId);

    this.subs.add(
      this.viewModel.runExperiment(this.experiment.id, value.controllerId).pipe(
        finalize(() => this.leaveProcessingState()),
        catchError(error => {
          if (error instanceof ErrorMessage) {
            this.service.alertFail(error.message.localized);
          }

          return Promise.reject(error);
        })
      )
        .subscribe(job => {
          this.jobs.items.unshift([job, controller]);
          this.service.alertSuccess('Experiment is queued to run');
          this.formGroup.reset({controllerId: ''});
        })
    );
  }
}
