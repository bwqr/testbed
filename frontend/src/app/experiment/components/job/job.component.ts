import {Component, ElementRef, OnInit, ViewChild} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {Job, JobStatus, SlimRunner} from '../../models';
import {ExperimentViewModelService} from '../../services/experiment-view-model.service';
import {ActivatedRoute} from '@angular/router';
import {switchMap} from 'rxjs/operators';
import {basicSetup, EditorState, EditorView} from '@codemirror/basic-setup';
import {python} from '@codemirror/lang-python';
import {formats} from '../../../../defs';
import {environment} from '../../../../environments/environment';
import {AuthService} from '../../../auth/services/auth.service';


@Component({
  selector: 'app-job',
  templateUrl: './job.component.html',
  styleUrls: ['./job.component.scss']
})
export class JobComponent extends MainComponent implements OnInit {

  job: Job;
  outputLink: string;

  jobStatuses = JobStatus;

  runner: SlimRunner;

  @ViewChild('code') code: ElementRef;

  editor: EditorView;

  formats = formats;

  get isPageReady(): boolean {
    return !!this.job && !!this.runner;
  }

  constructor(
    private viewModel: ExperimentViewModelService,
    private authService: AuthService,
    private activatedRoute: ActivatedRoute
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.activatedRoute.params.pipe(
        switchMap(params => this.viewModel.job(params.id))
      ).subscribe(([job, runner]) => {
        this.job = job;
        this.outputLink = `${environment.apiEndpoint}/experiment/job/${job.id}/output?token=${this.authService.getToken()}`;
        this.runner = runner;

        // experiment.code is html encoded, we need to decode it
        const el = document.createElement('textarea');
        el.innerHTML = this.job.code;
        const renderedCode = el.textContent;

        // experiment.code is html encoded, we need to decode it
        this.editor = new EditorView({
          state: EditorState.create({
            doc: renderedCode,
            extensions: [
              basicSetup,
              python(),
              EditorView.editable.of(false)
            ]
          }),
          parent: this.code.nativeElement
        });
      })
    );
  }

}
